//! #221: Every compute command of the user interface answers nonsense arguments with a result or an error message: no
//! crash in any thread (also Rayon), no hang, no invalid numbers. The commands come from the table
//! `crates/app/ipc-commands.json` (key `compute`), so a new command is included without further ado (AK4).
//!
//! Every call first writes command and arguments to stderr: if the process aborts (memory, stack) or hangs (time limit of
//! the nextest profile `ci`), the last output names the call. Number of cases via `PROPTEST_CASES` like proptest (see
//! [`cases`]: at a push about 130 calls, nightly all systematic cases and 625 random ones).

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Mutex, Once};
use std::time::Duration;

use engine::montecarlo::Rng;
use engine::{api, dispatch, progress};
use serde_json::{json, Value};

/// Panics in any thread since the start (the panic hook also sees Rayon workers, whose panic `catch_unwind` alone
/// would see only when it reaches the caller).
static PANICS: AtomicUsize = AtomicUsize::new(0);
static LAST_PANIC: Mutex<String> = Mutex::new(String::new());

fn count_panics() {
    static HOOK: Once = Once::new();
    HOOK.call_once(|| {
        let default = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            PANICS.fetch_add(1, Ordering::SeqCst);
            let thread = std::thread::current().name().unwrap_or("?").to_string();
            *LAST_PANIC.lock().unwrap_or_else(|e| e.into_inner()) = format!("Thread {thread}: {info}");
            default(info);
        }));
    });
}

fn table() -> Value {
    serde_json::from_str(include_str!("../../app/ipc-commands.json")).expect("Tabelle ist JSON")
}

/// Plan placeholders of the table, as in the contract test of the desktop app, but with fewer paths (short run).
fn resolve(args: &Value) -> Value {
    let mut a = args.clone();
    if let Some(p) = a.get("plan").and_then(Value::as_str) {
        let mut plan = api::example_plan();
        plan["scenarios"]["sweeps"] = json!([]);
        match p {
            "$plan" => plan["returns"]["source"] = json!("fixed"),
            "$planMc" => {
                plan["returns"]["source"] = json!("bootstrap");
                plan["returns"]["bootstrap"] = json!({ "paths": 20, "block": 12, "seed": 1 });
            }
            other => panic!("unbekannter Platzhalter {other}"),
        }
        a["plan"] = plan;
    }
    if let Some(r) = a.get_mut("request").and_then(Value::as_object_mut) {
        for key in ["paths", "checkPaths"] {
            if r.contains_key(key) {
                r.insert(key.into(), json!(10));
            }
        }
    }
    a
}

/// Commands of the table with their valid sample arguments (one entry per table row).
fn samples() -> Vec<(String, Value)> {
    table()["compute"]
        .as_array()
        .expect("compute")
        .iter()
        .map(|e| (e["cmd"].as_str().expect("cmd").to_string(), resolve(&e["args"])))
        .collect()
}

/// #221/AK1: Nonsense values: wrong type, `null`, empty object, extreme values, very long list.
fn nonsense() -> Vec<Value> {
    vec![
        Value::Null,
        json!(true),
        json!("x"),
        json!(""),
        json!(0),
        json!(-1),
        json!(0.5),
        json!(-1e308),
        json!(1e308),
        json!(u64::MAX),
        json!(i64::MIN),
        json!({}),
        json!([]),
        json!([[0, 3]]),
        Value::Array(vec![json!(1); 10_000]),
        Value::Array(vec![json!([0, 1]); 10_000]),
    ]
}

/// Pointers to all values below `v` (without the root).
fn pointers(v: &Value, path: String, out: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                let p = format!("{path}/{}", k.replace('~', "~0").replace('/', "~1"));
                out.push(p.clone());
                pointers(x, p, out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                let p = format!("{path}/{i}");
                out.push(p.clone());
                pointers(x, p, out);
            }
        }
        _ => {}
    }
}

fn set(args: &mut Value, pointer: &str, value: Option<Value>) {
    let (parent, key) = pointer.rsplit_once('/').expect("Zeiger");
    match (args.pointer_mut(parent), value) {
        (Some(Value::Object(o)), Some(v)) => {
            o.insert(key.replace("~1", "/").replace("~0", "~"), v);
        }
        (Some(Value::Object(o)), None) => {
            o.remove(&key.replace("~1", "/").replace("~0", "~"));
        }
        (Some(Value::Array(a)), v) => {
            if let Ok(i) = key.parse::<usize>() {
                if i < a.len() {
                    match v {
                        Some(v) => a[i] = v,
                        None => {
                            a.remove(i);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// Shortened arguments for the messages (a plan or a long list would fill the screen): the plan as `"…"`.
fn brief(args: &Value) -> String {
    let mut shown = args.clone();
    if let Some(plan) = shown.get_mut("plan").filter(|p| p.is_object()) {
        *plan = json!("…");
    }
    let s = shown.to_string();
    if s.len() > 300 {
        let cut = (0..=300).rev().find(|&i| s.is_char_boundary(i)).unwrap_or(0);
        format!("{}… ({} Zeichen)", &s[..cut], s.len())
    } else {
        s
    }
}

/// Longest computation of one call before the test cancels it like the user interface (button «Abbrechen»).
const CANCEL_AFTER: Duration = Duration::from_secs(5);

/// Runs `f` and cancels a running computation after [`CANCEL_AFTER`]: arguments with which a command computes as
/// intended (an empty request means the defaults of the optimization) would otherwise take minutes. A computation that
/// ignores the cancellation still ends in the time limit of the nextest profile `ci` (AK3).
fn with_watchdog<T>(f: impl FnOnce() -> T) -> T {
    let (done, wait) = mpsc::channel::<()>();
    let watchdog = std::thread::spawn(move || {
        if wait.recv_timeout(CANCEL_AFTER) == Err(mpsc::RecvTimeoutError::Timeout) {
            eprintln!("#221 abgebrochen nach {CANCEL_AFTER:?}");
            progress::cancel();
        }
    });
    let out = f();
    let _ = done.send(());
    watchdog.join().expect("Wächter");
    out
}

/// One call; a panic in any thread is an error that names command, change and arguments (AK2).
fn call(case: &Case, fallback: &Path) -> Result<Result<Value, String>, String> {
    let Case { cmd, change, args } = case;
    let what = format!("{cmd} [{change}] {}", brief(args));
    eprintln!("#221 {what}");
    let before = PANICS.load(Ordering::SeqCst);
    let out = with_watchdog(|| catch_unwind(AssertUnwindSafe(|| dispatch::call(cmd, args, &library(), fallback))));
    let after = PANICS.load(Ordering::SeqCst);
    if out.is_err() || after != before {
        let last = LAST_PANIC.lock().unwrap_or_else(|e| e.into_inner()).clone();
        return Err(format!("{what}: Absturz ({last})"));
    }
    Ok(out.unwrap().map_err(|m| m.text().to_string()))
}

/// One call of the sweep: command, description of the change against the sample arguments, arguments.
struct Case {
    cmd: String,
    change: String,
    args: Value,
}

/// Short form of a nonsense value for the description (a long list as its length).
fn label(v: &Option<Value>) -> String {
    match v {
        None => "gelöscht".into(),
        Some(Value::Array(a)) if a.len() > 3 => format!("Liste mit {} Einträgen", a.len()),
        Some(v) => v.to_string(),
    }
}

fn changed(cmd: &str, args: &Value, changes: &[(String, Option<Value>)]) -> Case {
    let mut a = args.clone();
    for (p, v) in changes {
        set(&mut a, p, v.clone());
    }
    let change = changes
        .iter()
        .map(|(p, v)| format!("{p}={}", label(v)))
        .collect::<Vec<_>>()
        .join(", ");
    Case {
        cmd: cmd.into(),
        change,
        args: a,
    }
}

/// Systematic cases per table row: whole argument object replaced, each field (also within objects except the plan)
/// deleted or replaced by every nonsense value; quantity fields of the plan with extreme values.
fn systematic() -> Vec<Case> {
    let mut cases = vec![];
    for (cmd, args) in samples() {
        for whole in [Value::Null, json!({}), json!([]), json!("x"), json!(1)] {
            cases.push(Case {
                cmd: cmd.clone(),
                change: format!("Argumente={whole}"),
                args: whole,
            });
        }
        let mut ptrs = vec![];
        pointers(&args, String::new(), &mut ptrs);
        ptrs.retain(|p| !p.starts_with("/plan/"));
        for p in &ptrs {
            cases.push(changed(&cmd, &args, &[(p.clone(), None)]));
            for v in nonsense() {
                cases.push(changed(&cmd, &args, &[(p.clone(), Some(v))]));
            }
        }
        if args.get("plan").is_some() {
            for (p, v) in [
                ("/plan/returns/bootstrap/paths", json!(u64::MAX)),
                ("/plan/returns/bootstrap/paths", json!(1_000_000)),
                ("/plan/returns/bootstrap/block", json!(1e308)),
                ("/plan/scenarios/sweeps", sweeps_long()),
            ] {
                cases.push(changed(&cmd, &args, &[(p.into(), Some(v))]));
            }
        }
    }
    cases
}

/// Variations with many values (a long list in the plan).
fn sweeps_long() -> Value {
    json!([
        { "param": "returns.fixed.stocks", "values": vec![json!(0.05); 2000] },
        { "param": "withdrawal.spending", "values": vec![json!(1); 2000] },
    ])
}

/// Random cases: one to three places of the sample arguments (also in the plan) replaced or deleted; commands without
/// arguments are covered by the systematic cases.
fn random(n: usize) -> Vec<Case> {
    let mut samples = samples();
    samples.retain(|(_, args)| args.as_object().is_some_and(|o| !o.is_empty()));
    let values = nonsense();
    let mut rng = Rng::new(221, 0);
    (0..n)
        .map(|_| {
            let (cmd, args) = &samples[rng.below(samples.len())];
            let mut ptrs = vec![];
            pointers(args, String::new(), &mut ptrs);
            let changes: Vec<(String, Option<Value>)> = (0..1 + rng.below(3))
                .map(|_| {
                    let p = ptrs[rng.below(ptrs.len())].clone();
                    let v = (rng.below(5) > 0).then(|| values[rng.below(values.len())].clone());
                    (p, v)
                })
                .collect();
            changed(cmd, args, &changes)
        })
        .collect()
}

/// #221/AK6: Budget like proptest from `PROPTEST_CASES` (default 256, nightly 10'000).
fn budget() -> usize {
    std::env::var("PROPTEST_CASES")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(256)
}

/// Cases of one run: per command the replaced whole argument object (cheap, every command is called), of the other
/// systematic cases `budget / 8` drawn without repetition (all of them nightly), and `budget / 16` random cases.
fn cases(budget: usize) -> Vec<Case> {
    let (mut chosen, mut rest): (Vec<Case>, Vec<Case>) = systematic()
        .into_iter()
        .partition(|c| c.change.starts_with("Argumente="));
    let mut rng = Rng::new(221, 1);
    for _ in 0..(budget / 8).min(rest.len()) {
        chosen.push(rest.swap_remove(rng.below(rest.len())));
    }
    chosen.extend(random(budget / 16));
    chosen
}

/// #221/AK1, AK2, AK6: Every command of the table with nonsense arguments ends with a result or an error message;
/// a panic in any thread fails the test and names command, change and arguments.
// slow: calls every command with many nonsense arguments
#[test]
fn issue_221_ak1_every_command_survives_nonsense_arguments() {
    count_panics();
    let dir = tempfile::Builder::new().prefix("vivaplan-").tempdir().unwrap();
    let cases = cases(budget());
    eprintln!("#221 {} Aufrufe", cases.len());
    let mut failures = vec![];
    for case in &cases {
        let t0 = std::time::Instant::now();
        if let Err(e) = call(case, dir.path()) {
            failures.push(e);
        }
        let ms = t0.elapsed().as_millis();
        if ms > 1000 {
            eprintln!("#221 langsam: {ms} ms");
        }
    }
    assert!(
        failures.is_empty(),
        "{} Abstürze:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// #221/AK6: Few cases at a push, nightly all systematic ones and many random ones.
// slow: builds every systematic case of the command sweep
#[test]
fn issue_221_ak6_number_of_cases_follows_the_budget() {
    let all = systematic().len();
    let commands = samples().len() * 5;
    assert_eq!(cases(256).len(), commands + 32 + 16);
    assert_eq!(cases(10_000).len(), all + 625);
    assert!(all > 4 * (commands + 48), "{all}");
}

/// #221/AK4: The sweep calls every compute command of the table, also one added later.
#[test]
fn issue_221_ak4_every_command_of_the_table_is_called() {
    let mut called: Vec<String> = systematic().into_iter().map(|c| c.cmd).collect();
    let mut expected: Vec<String> = samples().into_iter().map(|(c, _)| c).collect();
    for list in [&mut called, &mut expected] {
        list.sort();
        list.dedup();
    }
    assert_eq!(called, expected);
}

/// #221/AK5: With valid arguments every command returns a result, and that result has no NaN or ∞ (serde_json would write
/// them as `null`; the dispatcher therefore checks the numbers before converting and reports an error).
// slow: runs every compute command with valid arguments
#[test]
fn issue_221_ak5_valid_arguments_give_finite_results() {
    count_panics();
    let dir = tempfile::Builder::new().prefix("vivaplan-").tempdir().unwrap();
    for (cmd, args) in samples() {
        let case = Case {
            cmd,
            change: "gültig".into(),
            args,
        };
        let out = call(&case, dir.path()).unwrap_or_else(|e| panic!("{e}"));
        assert!(out.is_ok(), "{} {}: {out:?}", case.cmd, brief(&case.args));
    }
}

/// #221/AK1: Quantities that would compute for hours or exhaust the memory are rejected with a message; a plan that is
/// not an object is an error, not a crash.
#[test]
fn issue_221_ak1_quantity_limits_give_a_message() {
    count_panics();
    let dir = tempfile::Builder::new().prefix("vivaplan-").tempdir().unwrap();
    let sample = |cmd: &str| samples().into_iter().find(|(c, _)| c == cmd).unwrap().1;
    let run = |cmd: &str, change: &str, args: Value| {
        let case = Case {
            cmd: cmd.into(),
            change: change.into(),
            args,
        };
        call(&case, dir.path()).unwrap_or_else(|e| panic!("{e}"))
    };
    let mut rerun = sample("rerun_paths");
    rerun["picks"] = Value::Array(vec![json!([0, 1]); engine::api::MAX_PICKS + 1]);
    let out = run("rerun_paths", "101 Läufe", rerun).unwrap();
    assert_eq!(out["error"]["code"], "rerun_too_many_picks", "{out}");

    for (field, n, expected) in [
        ("samples", json!(u64::MAX), "at most 20'000'000 are possible"),
        ("checkPaths", json!(u64::MAX), "at most 20'000'000 are possible"),
        ("paths", json!(1_000_000), "Too many paths for the available memory"),
        (
            "checkPaths",
            json!(1_000_000),
            "Too many paths for the available memory",
        ),
    ] {
        let mut opt = sample("optimize");
        opt["request"][field] = n;
        let err = run("optimize", field, opt).unwrap_err();
        assert!(err.contains(expected) && !err.contains("-1"), "{field}: {err}");
    }
    let mut twice = sample("optimize");
    twice["request"]["strategies"] = json!(["fixed_rate", "fixed_rate", "fixed_rate"]);
    let out = run("optimize", "Strategie dreifach", twice).unwrap();
    assert_eq!(out["outcomes"].as_array().map(Vec::len), Some(1), "{}", brief(&out));

    for plan in [json!([[0, 3]]), json!(1), json!("x")] {
        let mut bench = sample("bench_plan");
        bench["plan"] = plan;
        assert!(run("bench_plan", "Planung kein Objekt", bench).is_err());
    }
}
