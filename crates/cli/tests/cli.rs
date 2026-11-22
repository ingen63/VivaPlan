//! Interface tests: all command line commands and the local development server.

// #116: memory limit per test process (engine::test_alloc).
#[global_allocator]
static TEST_ALLOC: engine::test_alloc::Limited = engine::test_alloc::Limited;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_vivaplan"))
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("pp-cli-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run(args: &[&str]) -> Output {
    bin().args(args).current_dir(root()).output().unwrap()
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// Example plan with adjusted returns, as a file.
fn plan_file(dir: &Path, source: &str, extra: impl Fn(&mut serde_json::Value)) -> String {
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root().join("crates/engine/assets/example.vplan")).unwrap())
            .unwrap();
    v["returns"]["source"] = serde_json::json!(source);
    v["returns"]["bootstrap"] = serde_json::json!({ "paths": 200, "block": 12, "seed": 1 });
    v["returns"]["parametric"] = serde_json::json!({ "paths": 50, "mu": 0.07, "sigma": 0.15, "df": 6 });
    extra(&mut v);
    let p = dir.join(format!("{source}.vplan"));
    std::fs::write(&p, v.to_string()).unwrap();
    p.to_string_lossy().into_owned()
}

#[test]
fn usage_and_schema() {
    let o = run(&[]);
    assert_eq!(o.status.code(), Some(2));
    assert!(text(&o).contains("Usage"));
    // #61: short form of the disclaimer in one line; #143/AK7: names tax and investment advice
    assert!(text(&o).contains("Model calculation without guarantee. No tax or investment advice. Taxes are estimates."));
    let o = run(&["schema"]);
    assert!(o.status.success());
    let schema: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(schema["properties"]["returns"].is_object());
}

// #93/AK2: `vivaplan --version` prints the workspace version (Cargo.toml, `version = "…"` under `[workspace.package]`).
#[test]
fn version_is_printed() {
    let ws = std::fs::read_to_string(root().join("Cargo.toml")).unwrap();
    let want = ws
        .lines()
        .find_map(|l| l.strip_prefix("version = \""))
        .unwrap()
        .trim_end_matches('"')
        .to_string();
    for flag in ["--version", "-V"] {
        let o = run(&[flag]);
        assert!(o.status.success(), "{flag}");
        assert_eq!(text(&o).trim(), format!("vivaplan {want}"), "{flag}");
    }
}

#[test]
fn run_single_with_outputs() {
    let d = tmp("single");
    let p = plan_file(&d, "fixed", |_| {});
    let (csv, json) = (d.join("m.csv"), d.join("r.json"));
    let o = run(&[
        "run",
        &p,
        "--events",
        "--csv",
        csv.to_str().unwrap(),
        "--json",
        json.to_str().unwrap(),
    ]);
    let t = text(&o);
    assert!(o.status.success(), "{t}");
    assert!(
        t.contains("Wealth at the end") && t.contains("Events:") && t.contains("(age "),
        "{t}"
    );
    let rows = std::fs::read_to_string(&csv).unwrap();
    assert!(rows.lines().count() > 300);
    // #143/AK1: the export names the taxes «estimated»; the values stay exact (AK2)
    assert!(
        rows.lines().next().unwrap().ends_with(",spending,taxes_estimated"),
        "{rows}"
    );
    assert!(t.contains("Estimated taxes total:"), "{t}");
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    assert!(r["summary"]["endTotal"].is_number());
}

#[test]
fn run_batches() {
    let d = tmp("batch");
    for src in ["rolling", "bootstrap", "parametric", "historical"] {
        let p = plan_file(&d, src, |_| {});
        let json = d.join(format!("{src}.json"));
        let o = run(&["run", &p, "--json", json.to_str().unwrap()]);
        let t = text(&o);
        assert!(o.status.success(), "{src}: {t}");
        if src != "historical" {
            assert!(t.contains("runs") && t.contains("percentile"), "{src}: {t}");
            assert!(std::fs::read_to_string(&json).unwrap().contains("successRate"));
        }
    }
}

#[test]
fn validate_and_errors() {
    let d = tmp("validate");
    let ok = plan_file(&d, "fixed", |_| {});
    let o = run(&["validate", &ok]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("valid"));
    // Without a series and without an investment strategy (#69), the returns for `rolling` are missing.
    let bad = plan_file(&d, "rolling", |v| {
        v["returns"]["series"] = serde_json::Value::Null;
        v["returns"].as_object_mut().unwrap().remove("strategy");
    });
    let o = run(&["validate", &bad]);
    assert!(!o.status.success());
    assert!(text(&o).contains("ERROR"));
    let o = run(&["run", "does-not-exist.vplan"]);
    assert!(!o.status.success());
    assert!(text(&o).contains("Error"));
    let o = run(&["validate"]);
    assert!(text(&o).contains("Project file missing"));
}

#[test]
fn bench_runs_paths() {
    let d = tmp("bench");
    let p = plan_file(&d, "fixed", |_| {});
    let o = run(&["bench", &p, "--paths", "200"]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("200 runs"));
}

#[test]
fn import_legacy_config() {
    let d = tmp("legacy");
    let out = d.join("legacy.vplan");
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../engine/tests/fixtures/legacy_fixed.json");
    let fixtures = fixture.parent().unwrap().to_str().unwrap().to_string();
    let o = run(&[
        "import-legacy",
        fixture.to_str().unwrap(),
        "--start-year",
        "2026",
        "--base",
        &fixtures,
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(o.status.success(), "{}", text(&o));
    let o = run(&["validate", out.to_str().unwrap()]);
    assert!(o.status.success(), "{}", text(&o));
    let o = run(&["import-legacy", fixture.to_str().unwrap()]);
    assert!(text(&o).contains("--start-year"));
}

#[test]
fn library_import_commands() {
    let lib = tmp("library");
    let l = lib.to_str().unwrap();
    let o = run(&["library", "--library", l]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("Shiller CHF.csv") && text(&o).contains("CHF"));
    // #94: the original table exists only locally; without it this part is skipped.
    let shiller = root().join("crates/engine/data/quellen/shiller-index.ods");
    if shiller.exists() {
        let o = run(&["import-shiller", shiller.to_str().unwrap(), "--library", l]);
        assert!(o.status.success(), "{}", text(&o));
        assert!(
            text(&o).contains("Written: Shiller CHF + CH-Obligationen.csv"),
            "{}",
            text(&o)
        );
    } else {
        eprintln!("skipped: {} missing (#94)", shiller.display());
    }
    let cube = root().join("crates/engine/data/quellen/snb-zimoma.csv");
    let o = run(&["import-snb", cube.to_str().unwrap(), "--library", l]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("zimoma imported"));
    let o = run(&["import-snb", "--library", l]);
    assert!(text(&o).contains("SNB file missing"));
    let o = run(&["import-shiller", "--library", l]);
    assert!(text(&o).contains("Shiller file missing"));
}

// ---------- Development server ----------

struct Server(Child, u16);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn server() -> Server {
    let port = 20_000 + (std::process::id() % 20_000) as u16;
    let srv = Server(
        bin()
            .args(["ui-dev", "--port", &port.to_string()])
            .current_dir(root())
            .stdout(Stdio::null())
            .spawn()
            .unwrap(),
        port,
    );
    // #83: wait for the state (port open) instead of a fixed delay; up to 30 s, so that a slow runner
    // under coverage measurement can start the server too.
    for _ in 0..600 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return srv;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("development server does not start");
}

fn http(port: u16, method: &str, path: &str, body: &str) -> (String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(60))).unwrap();
    write!(
        s,
        "{method} {path} HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
    .unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    let (head, body) = out.split_once("\r\n\r\n").unwrap_or((&out, ""));
    (head.to_string(), body.to_string())
}

#[test]
fn dev_server_serves_ui_and_api() {
    let srv = server();
    let p = srv.1;
    let (h, b) = http(p, "GET", "/", "");
    assert!(h.contains("200") && h.contains("text/html") && b.contains("VivaPlan"));
    for f in ["/app.js", "/charts.js", "/api.js", "/util.js"] {
        let (h, _) = http(p, "GET", f, "");
        assert!(h.contains("text/javascript"), "{f}: {h}");
    }
    let (h, _) = http(p, "GET", "/../Cargo.toml", "");
    assert!(h.contains("404"));
    let (_, plan) = http(p, "POST", "/api/example_plan", "{}");
    let args = format!("{{\"plan\": {plan}}}");
    let (h, b) = http(p, "POST", "/api/validate", &args);
    assert!(h.contains("200") && b.starts_with('['), "{h}");
    let (_, b) = http(p, "POST", "/api/preview", &args);
    assert!(b.contains("\"result\""));
    let (_, b) = http(p, "POST", "/api/run_simulation", &args);
    assert!(b.contains("\"single\"") || b.contains("\"batch\""));
    let args_r = format!("{{\"plan\": {plan}, \"picks\": [[50, 0]]}}");
    let (_, b) = http(p, "POST", "/api/rerun_paths", &args_r);
    assert!(b.contains("\"details\""), "{b}");
    let (_, b) = http(p, "POST", "/api/bench_plan", &args);
    assert!(b.contains("\"usPerRun\"") && b.contains("\"usGenerate\""), "{b}");
    let (_, b) = http(p, "POST", "/api/series_info", r#"{"path": "library:Shiller CHF.csv"}"#);
    assert!(b.contains("\"currency\":\"CHF\""), "{b}");
    let (h, b) = http(p, "POST", "/api/series_info", "{}");
    assert!(h.contains("400") && b.contains("Path missing"));
    let (_, b) = http(p, "POST", "/api/library_list", "{}");
    assert!(b.contains("Shiller USD.csv"));
    let (h, b) = http(p, "POST", "/api/open_plan", "{}");
    assert!(h.contains("400") && b.contains("desktop app"));
}

#[test]
fn optimize_compares_strategies_and_respects_ruin_limit() {
    let d = tmp("optimize");
    let f = plan_file(&d, "parametric", |_| {});
    let json = d.join("opt.json");
    let o = run(&[
        "optimize",
        &f,
        "--strategy",
        "fixed_rate,guardrails,vpw,endowment",
        "--paths",
        "100",
        "--check-paths",
        "200",
        "--samples",
        "4",
        "--json",
        json.to_str().unwrap(),
    ]);
    let t = text(&o);
    assert!(o.status.success(), "{t}");
    assert!(
        t.contains("Starting point") && t.contains("Fixed rate") && t.contains("Guardrails"),
        "{t}"
    );
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    assert_eq!(r.as_array().unwrap().len(), 4);
    for s in r.as_array().unwrap() {
        assert!(s["validated"]["ruin"].as_f64().unwrap() <= 1.0);
        assert!(s["validated"]["median"].as_f64().unwrap() >= 0.0);
    }
    assert!(!run(&["optimize", &f, "--strategy", "nonsense"]).status.success());
    assert!(!run(&["optimize"]).status.success());
}

#[test]
fn optimize_uses_saved_settings_and_options_override_them() {
    let d = tmp("optimize-saved");
    let f = plan_file(&d, "parametric", |p| {
        p["optimization"] = serde_json::json!({
            "maxRuin": 0.02, "ruinAge": 90, "paths": 100, "checkPaths": 200, "samples": 2, "strategies": ["fixed_rate"]
        });
    });
    let t = text(&run(&["optimize", &f]));
    assert!(t.contains("ruin ≤ 2.0 % until age 90"), "{t}");
    assert!(t.contains("Fixed rate") && !t.contains("Guardrails"), "{t}");
    let t = text(&run(&["optimize", &f, "--max-ruin", "8", "--strategy", "vpw"]));
    assert!(
        t.contains("ruin ≤ 8.0 %") && t.contains("VPW") && !t.contains("Fixed rate"),
        "{t}"
    );
}

#[test]
fn optimize_goal_two_needs_a_property_and_is_selectable() {
    let d = tmp("optimize-goal");
    let settings = serde_json::json!({ "paths": 100, "checkPaths": 100, "samples": 1, "strategies": ["fixed_rate"] });
    let f = plan_file(&d, "parametric", |p| p["optimization"] = settings.clone());
    // Goal 2 with the example plan's property
    let t = text(&run(&["optimize", &f, "--goal", "2", "--min-consumption", "50"]));
    assert!(
        t.contains("Goal 2 “No emergency sale of real estate”") && t.contains("unplanned property sale"),
        "{t}"
    );
    // Invalid goal
    assert!(!run(&["optimize", &f, "--goal", "9"]).status.success());
    // Goal 1 chosen explicitly: as before
    let t = text(&run(&["optimize", &f, "--goal", "1"]));
    assert!(t.contains("Goal: ruin"), "{t}");
    // Without a property: error message
    let g = plan_file(&d, "bootstrap", |p| {
        p["optimization"] = settings.clone();
        p["realestate"]["properties"] = serde_json::json!([]);
    });
    assert!(!run(&["optimize", &g, "--goal", "2"]).status.success());
}

// #144/AK3: `run` and `validate` reject a file from a newer version (non-zero exit code, message with the
// format id); the current and the old format id and a file without one calculate as before.
#[test]
fn issue_144_ak3_cli_rejects_an_unknown_format() {
    let d = tmp("format");
    let newer = plan_file(&d, "fixed", |v| v["format"] = serde_json::json!("vivaplan/2"));
    for cmd in ["run", "validate"] {
        let o = run(&[cmd, &newer]);
        assert!(!o.status.success(), "{cmd}: {}", text(&o));
        assert!(
            text(&o).contains("The file has the format “vivaplan/2”. It requires a newer version of VivaPlan"),
            "{cmd}: {}",
            text(&o)
        );
    }
    for format in [Some("vivaplan/1"), Some("pensionsplaner/1"), None] {
        let ok = plan_file(&d, "fixed", |v| match format {
            Some(f) => v["format"] = serde_json::json!(f),
            None => {
                v.as_object_mut().unwrap().remove("format");
            }
        });
        let o = run(&["validate", &ok]);
        assert!(o.status.success(), "{format:?}: {}", text(&o));
    }
}

// #196/AK7: `run` and `optimize` calculate with the switch «Immobilien nie verkaufen» from the file.
#[test]
fn issue_196_ak7_cli_uses_never_sell_from_the_file() {
    let d = tmp("never-sell");
    let settings = serde_json::json!({ "paths": 100, "checkPaths": 100, "samples": 1, "strategies": ["fixed_rate"] });
    let file = |source: &str, on: bool| {
        let s = settings.clone();
        let p = plan_file(&d, source, move |p| {
            p["optimization"] = s.clone();
            p["realestate"]["neverSell"] = serde_json::json!(on);
        });
        let to = d.join(format!("{source}-{on}.vplan"));
        std::fs::rename(&p, &to).unwrap();
        to.to_string_lossy().into_owned()
    };
    // run: without the switch the house is sold, with it there is no sale and the wealth is depleted.
    let off = text(&run(&["run", &file("fixed", false), "--events"]));
    assert!(
        off.contains("Emergency sale Haus") || off.contains("Sale Haus"),
        "{off}"
    );
    let on = text(&run(&["run", &file("fixed", true), "--events"]));
    assert!(!on.contains("Sale Haus") && !on.contains("Emergency sale"), "{on}");
    assert!(
        on.contains("Wealth depleted") && on.contains("“Never sell properties” is switched on"),
        "{on}"
    );
    // optimize: no run with an emergency sale.
    let share = |on: bool| {
        let json = d.join(format!("opt-{on}.json"));
        let o = run(&["optimize", &file("parametric", on), "--json", json.to_str().unwrap()]);
        assert!(o.status.success(), "{}", text(&o));
        let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
        r[0]["validated"]["emergencyShare"].as_f64().unwrap()
    };
    assert!(share(false) > 0.0);
    assert_eq!(share(true), 0.0);
}

// #197/AK4: `optimize --compare` compares the strategies with the plan's values without search, on the plan's paths;
// `--property both` shows every strategy twice.
#[test]
fn issue_197_ak4_cli_compare_without_optimization() {
    let d = tmp("compare");
    let f = plan_file(&d, "parametric", |_| {});
    let json = d.join("compare.json");
    let o = run(&[
        "optimize",
        "--compare",
        &f,
        "--strategy",
        "standard,guardrails",
        "--property",
        "both",
        "--json",
        json.to_str().unwrap(),
    ]);
    let t = text(&o);
    assert!(o.status.success(), "{t}");
    assert!(
        t.contains("Comparison of strategies without optimization") && t.contains("50 paths (as in run)"),
        "{t}"
    );
    assert!(
        t.contains("Guardrails (as planned)") && t.contains("Guardrails (never sell properties)"),
        "{t}"
    );
    assert!(
        t.contains("End wealth") && t.contains("Plan values") && !t.contains("Optimization"),
        "{t}"
    );
    // #205/AK1: the share of months at the floor (or «–») per strategy
    assert!(t.contains("At floor"), "{t}");
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    let list = r.as_array().unwrap();
    assert_eq!(list.len(), 4);
    for s in list {
        assert_eq!(s["evaluations"], 1);
        assert_eq!(s["validated"]["endTotal"].as_array().unwrap().len(), 5);
    }
    assert_eq!(list.iter().filter(|s| s["neverSell"] == true).count(), 2);
    assert!(!run(&["optimize", &f, "--compare", "--property", "x"]).status.success());
}

// #199/AK4: `run` reports the runs with an unaffordable mortgage renewed in full (never sell) in the report and in
// the JSON; without the switch the line and the field are missing (#199/AK5).
#[test]
fn issue_199_ak4_cli_reports_unaffordable_renewals() {
    let d = tmp("unaffordable");
    let report = |on: bool| {
        let p = plan_file(&d, "bootstrap", |p| {
            p["realestate"]["neverSell"] = serde_json::json!(on)
        });
        let json = d.join(format!("run-{on}.json"));
        let o = run(&["run", &p, "--json", json.to_str().unwrap()]);
        assert!(o.status.success(), "{}", text(&o));
        let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
        (text(&o), r)
    };
    let (t, r) = report(true);
    let n = r["unaffordableRenewalRuns"].as_u64().unwrap();
    assert!(n > 0, "{r}");
    assert!(
        t.contains(&format!(
            "Unaffordable mortgage renewed in full (never sell) in {n} runs ("
        )),
        "{t}"
    );
    let (t, r) = report(false);
    assert!(r.get("unaffordableRenewalRuns").is_none(), "{r}");
    assert!(!t.contains("Unaffordable"), "{t}");
}

// #199/AK4: the report of a single run names the renewals kept only because of «Immobilien nie verkaufen».
#[test]
fn issue_199_ak4_cli_single_run_reports_unaffordable_renewals() {
    let d = tmp("unaffordable-single");
    let p = plan_file(&d, "fixed", |p| {
        p["realestate"]["neverSell"] = serde_json::json!(true);
        p["returns"]["fixed"]["stocks"] = serde_json::json!(-0.05);
    });
    let json = d.join("r.json");
    let o = run(&["run", &p, "--json", json.to_str().unwrap()]);
    let t = text(&o);
    assert!(o.status.success(), "{t}");
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    let n = r["summary"]["unaffordableRenewals"].as_u64().unwrap();
    assert!(
        t.contains(&format!("Unaffordable renewals kept:   {n} (never sell)")),
        "{t}"
    );
}

// #204/AK1: `optimize` and `optimize --compare` list floor and ceiling with the plan value for VPW and endowment.
#[test]
fn issue_204_ak1_cli_lists_the_limits_for_vpw_and_endowment() {
    let d = tmp("limits");
    let f = plan_file(&d, "parametric", |p| {
        p["withdrawal"]["floor"] = serde_json::json!(0.8);
        p["withdrawal"]["ceiling"] = serde_json::json!(1.25);
    });
    let limits = "Floor of living expenses 80.0 %, Ceiling of living expenses 125.0 %";
    let o = run(&["optimize", "--compare", &f, "--strategy", "vpw,endowment"]);
    let t = text(&o);
    assert!(o.status.success(), "{t}");
    assert_eq!(t.matches(limits).count(), 2, "{t}");
    let o = run(&[
        "optimize",
        &f,
        "--strategy",
        "vpw,endowment",
        "--paths",
        "50",
        "--check-paths",
        "50",
        "--samples",
        "2",
    ]);
    let t = text(&o);
    assert!(o.status.success(), "{t}");
    assert_eq!(t.matches(&format!("fixed: {limits}")).count(), 2, "{t}");
}

fn stdout_json(o: &Output) -> serde_json::Value {
    serde_json::from_slice(&o.stdout).unwrap_or_else(|e| panic!("{e}: {}", text(o)))
}

// #207/AK1: `run`, `optimize`, `validate` and `bench` print with `--json` a fixed format on the standard output (and
// nothing else there); `--json <file.json>` still saves the detailed result.
#[test]
fn issue_207_ak1_json_on_standard_output() {
    let d = tmp("json-out");
    let fixed = plan_file(&d, "fixed", |_| {});
    let o = run(&["run", &fixed, "--json"]);
    assert!(o.status.success(), "{}", text(&o));
    let v = stdout_json(&o);
    assert_eq!(v["format"], "vivaplan-run/1");
    assert!(v["single"]["years"]["rows"].as_array().unwrap().len() > 30);
    // `--json` in front of the plan is a switch, not a file name
    let v = stdout_json(&run(&["run", "--json", &fixed]));
    assert_eq!(v["format"], "vivaplan-run/1");
    let boot = plan_file(&d, "bootstrap", |_| {});
    let v = stdout_json(&run(&["run", &boot, "--json"]));
    assert_eq!(v["batch"]["runs"], 200);

    let o = run(&["validate", &fixed, "--json"]);
    assert!(o.status.success(), "{}", text(&o));
    assert_eq!(
        stdout_json(&o),
        serde_json::json!({ "format": "vivaplan-validate/1", "valid": true, "issues": [] })
    );
    let bad = plan_file(&d, "historical", |v| v["taxes"]["multiplier"] = serde_json::json!(-1));
    let o = run(&["validate", "--json", &bad]);
    assert!(!o.status.success());
    let v = stdout_json(&o);
    assert_eq!(v["valid"], false);
    assert_eq!(v["issues"][0]["field"], "taxes.multiplier");

    let o = run(&["bench", &fixed, "--paths", "100", "--json"]);
    let v = stdout_json(&o);
    assert_eq!(v["format"], "vivaplan-bench/1");
    assert_eq!(v["runs"], 100);
    assert!(v["usPerRun"].as_f64().unwrap() > 0.0);

    let param = plan_file(&d, "parametric", |_| {});
    let o = run(&[
        "optimize",
        &param,
        "--strategy",
        "fixed_rate",
        "--paths",
        "50",
        "--check-paths",
        "50",
        "--samples",
        "2",
        "--json",
    ]);
    let v = stdout_json(&o);
    assert_eq!(v["format"], "vivaplan-optimize/1");
    assert_eq!(v["outcomes"].as_array().unwrap().len(), 1);
    assert!(v["outcomes"][0]["validated"]["ruin"].is_number());

    // A run that fails: the error in the format and a failure exit code.
    let broken = d.join("broken.vplan");
    let mut p: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&fixed).unwrap()).unwrap();
    p["milestones"][1]["person"] = serde_json::json!("Z");
    std::fs::write(&broken, p.to_string()).unwrap();
    let o = run(&["run", broken.to_str().unwrap(), "--json"]);
    assert!(!o.status.success());
    assert!(stdout_json(&o)["error"]["text"].is_string());
}

// #207/AK2, AK3: `set` changes values by field path into a new file next to the plan; on an error nothing is written.
#[test]
fn issue_207_ak2_ak3_set_writes_a_new_file() {
    let d = tmp("set");
    let p = plan_file(&d, "fixed", |_| {});
    let before = std::fs::read_to_string(&p).unwrap();
    let o = run(&["set", &p, "wealth.stockShare@retire=0.4", "project.name=Neu"]);
    assert!(o.status.success(), "{}", text(&o));
    let first = d.join("fixed-1.vplan");
    assert_eq!(String::from_utf8_lossy(&o.stdout).trim(), first.to_str().unwrap());
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&first).unwrap()).unwrap();
    assert_eq!(v["project"]["name"], "Neu");
    assert_eq!(
        v["wealth"]["stockShare"]["rows"][1],
        serde_json::json!({ "from": "retire", "value": 0.4 })
    );
    assert_eq!(std::fs::read_to_string(&p).unwrap(), before);
    // The next free number, never overwriting
    let o = run(&["set", &p, "wealth.cash=1"]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(d.join("fixed-2.vplan").exists());

    // AK2: unknown path or wrong type: messages, no file
    let o = run(&[
        "set",
        &p,
        "wealth.cash=2",
        "wealth.stockShar=0.5",
        "wealth.bonds=\"viel\"",
    ]);
    let t = text(&o);
    assert!(!o.status.success());
    assert!(
        t.contains("Unknown field path “wealth.stockShar”") && t.contains("expected number"),
        "{t}"
    );
    assert!(t.contains("No change made"), "{t}");
    assert!(!d.join("fixed-3.vplan").exists());
    assert_eq!(std::fs::read_to_string(&p).unwrap(), before);
    let o = run(&["set", &p]);
    assert!(text(&o).contains("Changes missing"), "{}", text(&o));

    // AK3: `-o` to an existing file only with `--overwrite`, after a backup
    let out = first.to_str().unwrap();
    let o = run(&["set", &p, "wealth.cash=3", "-o", out]);
    assert!(!o.status.success());
    assert!(text(&o).contains("--overwrite"), "{}", text(&o));
    let old_first = std::fs::read_to_string(&first).unwrap();
    let o = run(&["set", &p, "wealth.cash=3", "-o", out, "--overwrite"]);
    assert!(o.status.success(), "{}", text(&o));
    assert_eq!(std::fs::read_to_string(d.join("fixed-1.vplan.bak")).unwrap(), old_first);
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&first).unwrap()).unwrap();
    assert_eq!(v["wealth"]["cash"], 3);
    // `--overwrite` without `-o` replaces the plan itself, after a backup
    let o = run(&["set", &p, "wealth.cash=4", "--overwrite"]);
    assert!(o.status.success(), "{}", text(&o));
    assert_eq!(std::fs::read_to_string(format!("{p}.bak")).unwrap(), before);
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    assert_eq!(v["wealth"]["cash"], 4);
    // A new file without `-o` does not need `--overwrite`; a new `-o` file neither
    let fresh = d.join("fresh.vplan");
    let o = run(&["set", &p, "wealth.cash=5", "-o", fresh.to_str().unwrap()]);
    assert!(o.status.success() && fresh.exists(), "{}", text(&o));
    // The validation of the changed plan is shown as notes; the file is written all the same.
    let o = run(&["set", &p, "taxes.multiplier=-1"]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("ERROR [taxes]"), "{}", text(&o));
}

// #207/AK4: `diff` lists per difference the field path, the old and the new value.
#[test]
fn issue_207_ak4_diff_of_two_plans() {
    let d = tmp("diff");
    let p = plan_file(&d, "fixed", |_| {});
    let o = run(&["set", &p, "household.persons.0.name=Max", "withdrawal.floor=0.8"]);
    assert!(o.status.success(), "{}", text(&o));
    let q = d.join("fixed-1.vplan");
    let o = run(&["diff", &p, q.to_str().unwrap()]);
    let t = text(&o);
    assert!(o.status.success(), "{t}");
    assert_eq!(
        String::from_utf8_lossy(&o.stdout),
        "household.persons.0.name: \"Erika Muster\" → \"Max\"\nwithdrawal.floor: 0 → 0.8\n"
    );
    let v = stdout_json(&run(&["diff", &p, q.to_str().unwrap(), "--json"]));
    assert_eq!(v["format"], "vivaplan-diff/1");
    assert_eq!(
        v["changes"][1],
        serde_json::json!({ "path": "withdrawal.floor", "old": 0, "new": 0.8 })
    );
    let o = run(&["diff", &p, &p]);
    assert!(text(&o).contains("No differences."));
    let o = run(&["diff", &p]);
    assert!(!o.status.success());
    assert!(text(&o).contains("Two project files expected"));
}
