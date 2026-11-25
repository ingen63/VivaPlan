//! #210: The MCP server `vivaplan mcp` as a process: messages line by line over the standard input, answers on the
//! standard output (as an AI program uses it).

// #116: memory limit per test process (engine::test_alloc).
#[global_allocator]
static TEST_ALLOC: engine::test_alloc::Limited = engine::test_alloc::Limited;

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{json, Value};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Data folder of the app below the home folder `home`, as the app determines it per operating system.
fn app_data_in(home: &Path) -> PathBuf {
    let base = if cfg!(target_os = "macos") {
        home.join("Library").join("Application Support")
    } else if cfg!(windows) {
        home.join("AppData").join("Roaming")
    } else {
        home.join(".local").join("share")
    };
    base.join("ch.vivaplan-app.desktop")
}

/// Data folder with settings `settings` and the series `file` in «Zeitreihen».
fn data_folder(dir: &Path, settings: &str, file: &str) {
    std::fs::create_dir_all(dir.join("Zeitreihen")).unwrap();
    std::fs::write(dir.join("einstellungen.json"), settings).unwrap();
    std::fs::copy(
        root().join("crates/engine/data/Shiller CHF.csv"),
        dir.join("Zeitreihen").join(file),
    )
    .unwrap();
}

/// A running server with home folder `home`.
struct Server {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
    next: u64,
}

impl Server {
    fn start(home: &Path, args: &[&str]) -> Server {
        let mut child = Command::new(env!("CARGO_BIN_EXE_vivaplan"))
            .arg("mcp")
            .args(args)
            .env("HOME", home)
            .env("APPDATA", home.join("AppData").join("Roaming"))
            .env("USERPROFILE", home)
            .env_remove("XDG_DATA_HOME")
            .current_dir(root())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        Server {
            child,
            input,
            output,
            next: 0,
        }
    }

    fn send(&mut self, line: &str) {
        writeln!(self.input, "{line}").unwrap();
        self.input.flush().unwrap();
    }

    fn read(&mut self) -> Value {
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        assert!(line.ends_with('\n'), "one answer per line: {line:?}");
        serde_json::from_str(&line).unwrap()
    }

    fn request(&mut self, method: &str, params: Value) -> Value {
        self.next += 1;
        let id = self.next;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }).to_string());
        let answer = self.read();
        assert_eq!(answer["id"], id);
        assert_eq!(answer["jsonrpc"], "2.0");
        answer
    }

    /// Calls a tool; the text of the result and whether it is an error.
    fn tool(&mut self, name: &str, args: Value) -> (String, bool) {
        let a = self.request("tools/call", json!({ "name": name, "arguments": args }));
        let r = &a["result"];
        assert_eq!(r["content"][0]["type"], "text", "{a}");
        (
            r["content"][0]["text"].as_str().unwrap().to_string(),
            r["isError"] == true,
        )
    }

    fn initialize(&mut self) -> Value {
        let a = self.request(
            "initialize",
            json!({ "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": { "name": "test", "version": "1" } }),
        );
        self.send(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
        a
    }

    /// Closes the input; the server ends without an error.
    fn close(mut self) {
        drop(self.input);
        let status = self.child.wait().unwrap();
        assert!(status.success(), "{status}");
        let mut rest = String::new();
        self.output.read_line(&mut rest).unwrap();
        assert!(rest.is_empty(), "no further output: {rest}");
    }
}

fn data(text: &str) -> Value {
    let json = text.split("\n\nModel calculation").next().unwrap();
    serde_json::from_str(json).unwrap_or_else(|e| panic!("{e}: {text}"))
}

/// #210/AK1: The server signs on over stdio (initialize), lists the tools and answers a tool call; the tool list is the
/// recorded one (crates/cli/tests/mcp-tools.json), so every change of the interface is noticed.
#[test]
fn issue_210_ak1_sign_on_tool_list_and_call_over_stdio() {
    let tmp = tempfile::tempdir().unwrap();
    let mut s = Server::start(tmp.path(), &[]);
    let init = s.initialize();
    assert_eq!(init["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(init["result"]["serverInfo"]["name"], "vivaplan");
    assert_eq!(init["result"]["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));
    let tools = s.request("tools/list", json!({}))["result"]["tools"].clone();
    let recorded = root().join("crates/cli/tests/mcp-tools.json");
    let expected: Value = serde_json::from_str(&std::fs::read_to_string(&recorded).unwrap()).unwrap();
    assert_eq!(
        tools,
        expected,
        "tool list changed; if intended, write this into {}:\n{}",
        recorded.display(),
        serde_json::to_string_pretty(&tools).unwrap()
    );
    let (text, error) = s.tool("plan_open", json!({ "example": true }));
    assert!(!error, "{text}");
    assert_eq!(data(&text)["plan"], "p1");
    // Broken input: an error of the protocol, the server keeps running.
    s.send("{kaputt");
    assert_eq!(s.read()["error"]["code"], -32700);
    assert_eq!(s.request("ping", json!({}))["result"], json!({}));
    s.close();
}

/// #210/AK2: The reading tools over stdio on a plan file: open, read, schema, check, results, data library, describe.
#[test]
fn issue_210_ak2_reading_tools_on_a_plan_file() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("Erika Muster").join("Pläne");
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("rente.vplan");
    std::fs::copy(root().join("crates/engine/assets/example.vplan"), &file).unwrap();
    let mut s = Server::start(
        tmp.path(),
        &["--library", root().join("crates/engine/data").to_str().unwrap()],
    );
    s.initialize();
    let (text, error) = s.tool("plan_open", json!({ "path": file }));
    assert!(!error, "{text}");
    let o = data(&text);
    assert_eq!(o["file"], "rente.vplan");
    assert_eq!(o["checks"]["errors"], 0);
    assert_eq!(o["period"]["start"], "2026-03");

    let (text, _) = s.tool("plan_get", json!({ "plan": "p1", "path": "wealth" }));
    assert!(data(&text)["value"].is_object(), "{text}");
    let (text, _) = s.tool("plan_schema", json!({ "section": "pillar3a" }));
    assert!(data(&text)["definitions"].is_object(), "{text}");
    let (text, _) = s.tool("plan_validate", json!({ "plan": "p1" }));
    assert_eq!(data(&text)["errors"], 0);
    let (text, _) = s.tool("result_list", json!({}));
    assert_eq!(data(&text)["results"], json!([]));
    let (text, error) = s.tool("result_get", json!({ "result": "r1" }));
    assert!(error && text.contains("no results yet"), "{text}");
    let (text, _) = s.tool("library_list", json!({}));
    assert!(text.contains("library:Shiller CHF.csv"), "{text}");
    let (text, _) = s.tool("describe", json!({ "section": "experiment", "plan": "p1" }));
    assert!(data(&text)["experiment"].is_object(), "{text}");
    s.close();
}

/// #210/AK4: Limits and data library come from the data folder of the app; --data-dir overrides it.
#[test]
fn issue_210_ak4_data_folder_of_the_app_and_data_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let home = tmp.path().join("home");
    data_folder(&app_data_in(&home), r#"{"limits": {"pillar3aMax": 2000}}"#, "App.csv");
    let other = tmp.path().join("other");
    data_folder(&other, r#"{"limits": {"pillar3aMax": 9000}}"#, "Other.csv");
    let mut plan: Value =
        serde_json::from_str(&std::fs::read_to_string(root().join("crates/engine/assets/example.vplan")).unwrap())
            .unwrap();
    plan["pillar3a"][0]["contribution"] = json!(7000);
    let file = tmp.path().join("p.vplan");
    std::fs::write(&file, plan.to_string()).unwrap();

    let check = |args: &[&str]| -> (Value, String) {
        let mut s = Server::start(&home, args);
        s.initialize();
        s.tool("plan_open", json!({ "path": file }));
        let (v, _) = s.tool("plan_validate", json!({ "plan": "p1" }));
        let (l, _) = s.tool("library_list", json!({}));
        s.close();
        (data(&v), l)
    };
    let (v, library) = check(&[]);
    let codes: Vec<&str> = v["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"pillar3a_contribution_range"), "{v}");
    assert!(v["messages"][0]["text"].as_str().unwrap().contains("2'000"), "{v}");
    assert!(
        library.contains("App.csv") && !library.contains("Other.csv"),
        "{library}"
    );
    let other = other.to_str().unwrap();
    let (v, library) = check(&["--data-dir", other]);
    assert_eq!(v["errors"], 0, "{v}");
    assert!(
        library.contains("Other.csv") && !library.contains("App.csv"),
        "{library}"
    );
}

/// #210/AK5: Answers name neither persons nor folders (unless --names); a file with another ending is not read.
#[test]
fn issue_210_ak5_no_names_and_no_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("hans.muster").join("Vorsorge");
    std::fs::create_dir_all(&dir).unwrap();
    let mut plan: Value =
        serde_json::from_str(&std::fs::read_to_string(root().join("crates/engine/assets/example.vplan")).unwrap())
            .unwrap();
    plan["household"]["persons"][0]["name"] = json!("Hans Muster");
    plan["project"]["name"] = json!("Pension Hans Muster");
    plan["household"]["persons"][0]["birth"] = json!("2030-01");
    let file = dir.join("hans.vplan");
    std::fs::write(&file, plan.to_string()).unwrap();
    std::fs::write(dir.join("secret.txt"), "x").unwrap();

    let mut s = Server::start(tmp.path(), &[]);
    s.initialize();
    let mut all = String::new();
    let (t, _) = s.tool("plan_open", json!({ "path": file }));
    all += &t;
    for args in [
        json!({ "plan": "p1", "section": "household" }),
        json!({ "plan": "p1", "section": "project" }),
    ] {
        all += &s.tool("plan_get", args).0;
    }
    all += &s.tool("plan_validate", json!({ "plan": "p1" })).0;
    all += &s.tool("plan_open", json!({ "path": dir.join("missing.vplan") })).0;
    let (t, error) = s.tool("plan_open", json!({ "path": dir.join("secret.txt") }));
    assert!(error && t.contains(".vplan"), "{t}");
    s.close();
    for word in [
        "Hans",
        "Muster",
        "hans.muster",
        "Vorsorge",
        tmp.path().to_str().unwrap(),
    ] {
        assert!(!all.contains(word), "{word} in {all}");
    }
    assert!(all.contains("Person 1") && all.contains("hans.vplan"), "{all}");

    let mut s = Server::start(tmp.path(), &["--names"]);
    s.initialize();
    s.tool("plan_open", json!({ "path": file }));
    let (t, _) = s.tool("plan_get", json!({ "plan": "p1", "path": "household.persons.0.name" }));
    assert!(t.contains("Hans Muster"), "{t}");
    s.close();
}

/// #210/AK5: Every tool stays below 20'000 characters, also with the whole catalog and the largest sections.
#[test]
fn issue_210_ak5_every_answer_below_the_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let mut s = Server::start(tmp.path(), &[]);
    s.initialize();
    s.tool("plan_open", json!({ "example": true }));
    let schema: Value = serde_json::from_str(&engine::schema_json()).unwrap();
    let mut calls = vec![
        ("describe", json!({})),
        ("plan_schema", json!({})),
        ("plan_get", json!({ "plan": "p1" })),
        ("library_list", json!({})),
        ("result_list", json!({})),
    ];
    for sec in schema["properties"].as_object().unwrap().keys() {
        calls.push(("plan_schema", json!({ "section": sec })));
        calls.push(("plan_get", json!({ "plan": "p1", "section": sec })));
    }
    for sec in [
        "fields",
        "series",
        "state",
        "metrics",
        "experiment",
        "examples",
        "limits",
    ] {
        calls.push(("describe", json!({ "section": sec })));
    }
    for (name, args) in calls {
        let (t, _) = s.tool(name, args.clone());
        assert!(t.chars().count() <= 20_000, "{name} {args}: {}", t.chars().count());
    }
    s.close();
}

/// #210/AK6: The first plan_open brings the note on privacy with «your AI and its provider» and the full disclaimer; the
/// description of the server names the disclaimer as well.
#[test]
fn issue_210_ak6_privacy_note_and_disclaimer() {
    let tmp = tempfile::tempdir().unwrap();
    let mut s = Server::start(tmp.path(), &[]);
    let init = s.initialize();
    let instructions = init["result"]["instructions"].as_str().unwrap();
    assert!(instructions.contains("without guarantee"), "{instructions}");
    let (t, _) = s.tool("plan_open", json!({ "example": true }));
    let o = data(&t);
    assert!(o["privacy"]
        .as_str()
        .unwrap()
        .contains("your AI and is processed by its provider"));
    assert!(o["disclaimer"]
        .as_str()
        .unwrap()
        .contains("not tax or investment advice"));
    let (t, _) = s.tool("plan_open", json!({ "example": true }));
    assert!(data(&t).get("privacy").is_none());
    let (t, _) = s.tool("result_list", json!({}));
    assert!(t.ends_with("Model calculation without guarantee. No tax or investment advice. Taxes are estimates."));
    s.close();
}

// ---------------------------------------------------------------------------------------------------------------------
// #211: Computing tools

const DISCLAIMER: &str = "Model calculation without guarantee. No tax or investment advice. Taxes are estimates.";

/// The example plan with the source of returns `source` and `paths` paths, written as a project file into `dir`.
fn plan_file(dir: &Path, name: &str, source: &str, paths: u64, sweeps: Value) -> (PathBuf, Value) {
    let mut plan = engine::api::example_plan();
    plan["returns"]["source"] = json!(source);
    plan["returns"]["bootstrap"]["paths"] = json!(paths);
    plan["scenarios"]["sweeps"] = sweeps;
    let file = dir.join(name);
    std::fs::write(&file, serde_json::to_string_pretty(&plan).unwrap()).unwrap();
    (file, plan)
}

/// A plan whose simulation takes far longer than 60 s, even in a release build (about 40 million runs).
fn long_plan(dir: &Path) -> PathBuf {
    let sweeps = json!([
        { "param": "flows.spendingFactor", "from": 0.5, "to": 1.5, "step": 0.01 },
        { "param": "wealth.stockShare", "from": 0.0, "to": 1.0, "step": 0.05 },
    ]);
    plan_file(dir, "long.vplan", "bootstrap", 20_000, sweeps).0
}

/// An experiment that takes far longer than 60 s, even in a release build (about 10 million runs).
fn long_experiment() -> Value {
    json!({
        "set": { "returns.source": "parametric" },
        "budget": { "paths": 5000 },
        "metrics": [{ "name": "ruin", "quantity": "depleted", "reduce": "any", "aggregate": { "kind": "probability" } }],
        "vary": [
            { "path": "flows.spendingFactor", "range": [0.5, 1.5, 0.01] },
            { "path": "pension.pk.lumpsum", "range": [0.0, 1.0, 0.05] },
        ],
    })
}

/// The example «lumpSum» of the catalog: three variants side by side, small.
fn small_experiment() -> Value {
    json!({
        "set": { "returns.source": "parametric" },
        "budget": { "paths": 300 },
        "metrics": [
            { "name": "ruin", "quantity": "depleted", "reduce": "any", "aggregate": { "kind": "probability" } },
            { "name": "end10", "quantity": "total", "real": true, "reduce": "atEnd", "aggregate": { "kind": "quantile", "q": 0.1 } },
        ],
        "vary": [{ "path": "pension.pk.lumpsum", "values": [0, 0.5, 1] }],
    })
}

impl Server {
    /// Starts a tool call with a progress token without waiting for the answer; the id of the request.
    fn start_call(&mut self, name: &str, args: Value, token: &str) -> u64 {
        self.next += 1;
        let id = self.next;
        let params = json!({ "name": name, "arguments": args, "_meta": { "progressToken": token } });
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": params }).to_string());
        id
    }

    /// Reads until the first message that is not a progress notification; the notifications read on the way.
    fn read_past_progress(&mut self) -> (Vec<Value>, Value) {
        let mut progress = vec![];
        loop {
            let m = self.read();
            if m["method"] == "notifications/progress" {
                progress.push(m);
            } else {
                return (progress, m);
            }
        }
    }

    fn cancel(&mut self, id: u64) {
        let msg = json!({ "jsonrpc": "2.0", "method": "notifications/cancelled", "params": { "requestId": id, "reason": "test" } });
        self.send(&msg.to_string());
    }
}

/// #211/AK1: simulation_run computes the plan as the app does and keeps the result for result_get; evaluate computes an
/// experiment; both answer with the disclaimer, errors of the input are tool errors.
// slow: starts the program and computes over stdio
#[test]
fn issue_211_ak1_simulation_run_and_evaluate() {
    let tmp = tempfile::tempdir().unwrap();
    let (file, _) = plan_file(tmp.path(), "plan.vplan", "bootstrap", 300, json!([]));
    let mut s = Server::start(tmp.path(), &[]);
    s.initialize();
    let (t, _) = s.tool("plan_open", json!({ "path": file.to_str().unwrap() }));
    assert_eq!(data(&t)["plan"], "p1");
    let (t, error) = s.tool("simulation_run", json!({ "plan": "p1", "paths": 200 }));
    assert!(!error, "{t}");
    assert!(t.ends_with(DISCLAIMER), "{t}");
    let r = data(&t);
    assert_eq!(r["result"], "r1");
    assert_eq!(r["digest"]["runs"], 200);
    assert!(r["verdict"].as_str().unwrap().contains("runs"), "{r}");
    assert_eq!(r["estimate"]["runs"], 200);
    assert!(r["estimate"]["seconds"].as_f64().unwrap() < 60.0);
    let (t, error) = s.tool("result_get", json!({ "result": "r1", "view": "bands" }));
    assert!(!error, "{t}");
    assert!(!data(&t)["rows"].as_array().unwrap().is_empty());

    let (t, error) = s.tool("evaluate", json!({ "plan": "p1", "experiment": small_experiment() }));
    assert!(!error, "{t}");
    assert!(t.ends_with(DISCLAIMER));
    let e = data(&t);
    assert_eq!((e["result"].as_str(), e["status"].as_str()), (Some("r2"), Some("done")));
    assert_eq!(e["experiment"]["listed"].as_array().unwrap().len(), 3);
    let (t, _) = s.tool("result_list", json!({}));
    let list = data(&t)["results"].clone();
    assert_eq!(list[0]["tool"], "simulation_run");
    assert_eq!(
        (list[1]["tool"].as_str(), list[1]["variants"].as_u64()),
        (Some("evaluate"), Some(3))
    );
    let (t, error) = s.tool("result_get", json!({ "result": "r2", "view": "yearly" }));
    assert!(error && t.contains("simulation_run"), "{t}");

    // Errors of the input come back as tool errors the AI can correct.
    let (t, error) = s.tool(
        "evaluate",
        json!({ "plan": "p1", "experiment": { "vary": [{ "path": "no.such.field", "values": [1] }] } }),
    );
    assert!(error, "{t}");
    let (t, error) = s.tool("evaluate", json!({ "plan": "p1", "experiment": [] }));
    assert!(error && t.contains("type object"), "{t}");
    let (t, error) = s.tool("simulation_run", json!({ "plan": "p9" }));
    assert!(error && t.contains("Unknown plan p9"), "{t}");
    let (t, _) = s.tool("plan_open", json!({ "example": true }));
    assert_eq!(data(&t)["plan"], "p2");
    let (t, error) = s.tool("simulation_run", json!({ "plan": "p2", "paths": 100 }));
    assert!(error && t.contains("bootstrap and parametric"), "{t}");
    let (t, error) = s.tool("simulation_run", json!({ "plan": "p2" }));
    assert!(!error, "{t}");
    assert!(data(&t)["verdict"].as_str().unwrap().starts_with("The wealth"), "{t}");
    s.close();
}

/// #211/AK2: A long simulation reports its progress about once a second; while it runs the server answers other
/// requests and refuses a second computation; the cancellation stops it without an answer to the request.
// slow: starts the program and computes over stdio
#[test]
fn issue_211_ak2_progress_and_cancel_of_simulation_run() {
    let tmp = tempfile::tempdir().unwrap();
    let file = long_plan(tmp.path());
    let mut s = Server::start(tmp.path(), &[]);
    s.initialize();
    s.tool("plan_open", json!({ "path": file.to_str().unwrap() }));
    let id = s.start_call("simulation_run", json!({ "plan": "p1", "confirm": true }), "sim");
    // Two notifications: the progress goes up and names the token.
    let (mut seen, mut last) = (vec![], -1.0);
    while seen.len() < 2 {
        let m = s.read();
        assert_eq!(m["method"], "notifications/progress", "{m}");
        assert_eq!(m["params"]["progressToken"], "sim");
        let p = m["params"]["progress"].as_f64().unwrap();
        assert!(p > last, "{m}");
        last = p;
        seen.push(m);
    }
    assert!(seen[1]["params"]["total"].as_f64().unwrap() >= last);
    // Meanwhile: reading tools answer, a second computation is refused.
    s.next += 1;
    let ping = s.next;
    s.send(&json!({ "jsonrpc": "2.0", "id": ping, "method": "ping" }).to_string());
    let (_, m) = s.read_past_progress();
    assert_eq!(m["id"], ping);
    let second = s.start_call("simulation_run", json!({ "plan": "p1" }), "second");
    let (_, m) = s.read_past_progress();
    assert_eq!(m["id"], second);
    assert_eq!(m["result"]["isError"], true);
    assert!(m["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .contains("Another computation is running"));
    // Cancel: no answer to the request, the next computation starts soon.
    let t0 = std::time::Instant::now();
    s.cancel(id);
    loop {
        let small = s.start_call("result_list", json!({}), "x");
        let (_, m) = s.read_past_progress();
        assert_eq!(m["id"], small, "no answer to the cancelled request: {m}");
        let next = s.start_call(
            "simulation_run",
            json!({ "plan": "p1", "paths": 10, "confirm": true }),
            "next",
        );
        let (_, m) = s.read_past_progress();
        assert_eq!(m["id"], next, "{m}");
        if m["result"]["isError"] != true {
            break;
        }
        assert!(t0.elapsed().as_secs() < 20, "the cancelled computation still runs: {m}");
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    s.close();
}

/// #211/AK2: evaluate reports its progress and is cancelled in the middle; the end of the input cancels a running
/// computation and ends the server.
// slow: starts the program and computes over stdio
#[test]
fn issue_211_ak2_progress_and_cancel_of_evaluate() {
    let tmp = tempfile::tempdir().unwrap();
    let mut s = Server::start(tmp.path(), &[]);
    s.initialize();
    s.tool("plan_open", json!({ "example": true }));
    let args = json!({ "plan": "p1", "experiment": long_experiment(), "confirm": true });
    let id = s.start_call("evaluate", args.clone(), 7.to_string().as_str());
    let m = s.read();
    assert_eq!(m["method"], "notifications/progress", "{m}");
    assert_eq!(m["params"]["progressToken"], "7");
    assert!(m["params"]["total"].as_f64().unwrap() > 1e6, "{m}");
    s.cancel(id);
    let (t, error) = loop {
        let next = s.start_call(
            "evaluate",
            json!({ "plan": "p1", "experiment": small_experiment() }),
            "n",
        );
        let (_, m) = s.read_past_progress();
        assert_eq!(m["id"], next, "no answer to the cancelled request: {m}");
        let text = m["result"]["content"][0]["text"].as_str().unwrap().to_string();
        if !text.contains("Another computation") {
            break (text, m["result"]["isError"] == true);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    };
    assert!(!error, "{t}");
    // The end of the input while computing: the server cancels and ends at once.
    s.start_call("evaluate", args, "end");
    s.read();
    let t0 = std::time::Instant::now();
    drop(s.input);
    let status = s.child.wait().unwrap();
    assert!(status.success() && t0.elapsed().as_secs() < 20, "{status}");
}

/// #211/AK3: From 60 s of estimated duration nothing is computed without confirm: true; the answer gives the estimate.
// slow: starts the program and measures the plan
#[test]
fn issue_211_ak3_confirmation_from_60_seconds() {
    let tmp = tempfile::tempdir().unwrap();
    let file = long_plan(tmp.path());
    let mut s = Server::start(tmp.path(), &[]);
    s.initialize();
    s.tool("plan_open", json!({ "path": file.to_str().unwrap() }));
    let t0 = std::time::Instant::now();
    let (t, error) = s.tool("simulation_run", json!({ "plan": "p1" }));
    assert!(!error, "{t}");
    let a = data(&t);
    assert_eq!(a["status"], "needsConfirmation");
    assert!(a["estimate"]["seconds"].as_f64().unwrap() >= 60.0, "{a}");
    assert_eq!(a["estimate"]["runs"], 20_000 * 101 * 21);
    assert!(a["next"].as_str().unwrap().contains("confirm: true"));
    assert!(!t.ends_with(DISCLAIMER), "no result: {t}");
    let (t, error) = s.tool("evaluate", json!({ "plan": "p1", "experiment": long_experiment() }));
    assert!(!error, "{t}");
    let a = data(&t);
    assert_eq!(a["status"], "needsConfirmation");
    assert!(a["estimate"]["seconds"].as_f64().unwrap() >= 60.0, "{a}");
    assert!(a["message"].as_str().unwrap().contains("confirm"), "{a}");
    // #258/AK1: compare with the estimate, which is measured live and slows down with the machine.
    let estimate = a["estimate"]["seconds"].as_f64().unwrap();
    assert!(t0.elapsed().as_secs_f64() < estimate, "nothing computed");
    let (t, _) = s.tool("result_list", json!({}));
    assert_eq!(data(&t)["results"], json!([]));
    s.close();
}

/// Equal up to the last digits of a number (reading JSON text rounds in the last place).
fn assert_close(a: &Value, b: &Value, at: &str) {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => {
            let (x, y) = (x.as_f64().unwrap(), y.as_f64().unwrap());
            assert!((x - y).abs() <= 1e-12 * x.abs().max(y.abs()), "{at}: {x} != {y}");
        }
        (Value::Array(x), Value::Array(y)) => {
            assert_eq!(x.len(), y.len(), "{at}");
            for (i, (x, y)) in x.iter().zip(y).enumerate() {
                assert_close(x, y, &format!("{at}.{i}"));
            }
        }
        (Value::Object(x), Value::Object(y)) => {
            assert_eq!(x.keys().collect::<Vec<_>>(), y.keys().collect::<Vec<_>>(), "{at}");
            for (k, x) in x {
                assert_close(x, &y[k], &format!("{at}.{k}"));
            }
        }
        _ => assert_eq!(a, b, "{at}"),
    }
}

/// #211/AK4: The key figures of simulation_run are those of api::run, the ranking of evaluate that of api::evaluate.
// slow: computes over stdio and in the test
#[test]
fn issue_211_ak4_key_figures_as_api_run_and_evaluate() {
    let tmp = tempfile::tempdir().unwrap();
    let (file, mut plan) = plan_file(tmp.path(), "plan.vplan", "bootstrap", 500, json!([]));
    let mut s = Server::start(tmp.path(), &[]);
    s.initialize();
    s.tool("plan_open", json!({ "path": file.to_str().unwrap() }));
    let (t, error) = s.tool("simulation_run", json!({ "plan": "p1" }));
    assert!(!error, "{t}");
    let answer = data(&t);
    let (t, error) = s.tool("evaluate", json!({ "plan": "p1", "experiment": small_experiment() }));
    assert!(!error, "{t}");
    let experiment = data(&t);
    s.close();

    // The same computation as the app: plan with the (empty) settings, library of the data folder.
    engine::appdata::apply(&mut plan, &serde_json::Map::new());
    let library = app_data_in(tmp.path()).join("Zeitreihen");
    let library = library.is_dir().then_some(library.as_path());
    let run = engine::api::run(&plan, tmp.path(), library);
    let digest = engine::api::digest(&serde_json::to_value(&run).unwrap()).unwrap();
    assert_close(&answer["digest"], &serde_json::to_value(digest).unwrap(), "digest");
    assert_eq!(answer["digest"]["runs"], 500);
    let direct = engine::api::evaluate(&plan, &small_experiment(), tmp.path(), library);
    let direct = serde_json::to_value(direct).unwrap();
    assert_close(
        &experiment["experiment"]["listed"],
        &direct["result"]["listed"],
        "listed",
    );
    assert_eq!(experiment["experiment"]["paths"], direct["result"]["paths"]);
}

// #212: Changing and saving

impl Server {
    /// Sign-on of a program that can ask the user (capability elicitation).
    fn initialize_asking(&mut self) {
        let a = self.request(
            "initialize",
            json!({ "protocolVersion": "2025-06-18", "capabilities": { "elicitation": {} }, "clientInfo": { "name": "test", "version": "1" } }),
        );
        assert!(a["result"].is_object(), "{a}");
        self.send(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#);
    }

    /// Calls plan_save with overwrite and answers the question of the server with `answer`; the result of the call.
    fn save_answering(&mut self, args: Value, answer: Value) -> (Value, String, bool) {
        self.next += 1;
        let id = self.next;
        let params = json!({ "name": "plan_save", "arguments": args });
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": "tools/call", "params": params }).to_string());
        let question = self.read();
        assert_eq!(question["method"], "elicitation/create", "{question}");
        let reply = json!({ "jsonrpc": "2.0", "id": question["id"], "result": answer });
        self.send(&reply.to_string());
        let a = self.read();
        assert_eq!(a["id"], id);
        let text = a["result"]["content"][0]["text"].as_str().unwrap().to_string();
        (question, text, a["result"]["isError"] == true)
    }
}

/// #212/AK1, AK2: Over stdio: change, list the changes, undo, take over a variant of evaluate, save to a new file and
/// replace the opened file only after the question of the server, with a backup.
// slow: starts the program and computes over stdio
#[test]
fn issue_212_ak1_ak2_change_undo_and_save_over_stdio() {
    let tmp = tempfile::tempdir().unwrap();
    let (file, plan) = plan_file(tmp.path(), "plan.vplan", "bootstrap", 300, json!([]));
    let original = std::fs::read(&file).unwrap();
    let mut s = Server::start(tmp.path(), &[]);
    s.initialize_asking();
    let (text, error) = s.tool("plan_open", json!({ "path": file.display().to_string() }));
    assert!(!error, "{text}");
    // Change, list, undo.
    let changes =
        json!([{ "path": "wealth.stockShare", "value": 0.25 }, { "path": "withdrawal.strategy", "value": "vpw" }]);
    let (text, error) = s.tool("plan_set", json!({ "plan": "p1", "changes": changes }));
    assert!(!error, "{text}");
    let set = data(&text);
    assert_eq!(set["changes"][0]["path"], "wealth.stockShare");
    assert_eq!(set["changes"][0]["old"], plan["wealth"]["stockShare"]);
    assert_eq!(set["changes"][0]["new"], json!(0.25));
    let (text, error) = s.tool(
        "plan_set",
        json!({ "plan": "p1", "changes": [{ "path": "wealth.nope", "value": 1 }] }),
    );
    assert!(error && text.starts_with("No change made"), "{text}");
    let (text, _) = s.tool(
        "plan_set",
        json!({ "plan": "p1", "changes": [{ "path": "wealth.stockShare", "value": 0.5 }] }),
    );
    assert_eq!(data(&text)["steps"], 2);
    let (text, _) = s.tool("plan_changes", json!({ "plan": "p1" }));
    let list = data(&text);
    assert_eq!(list["count"], 2);
    assert_eq!(list["changes"][0]["file"], plan["wealth"]["stockShare"]);
    assert_eq!(list["changes"][0]["current"], json!(0.5));
    let (text, _) = s.tool("plan_undo", json!({ "plan": "p1" }));
    assert_eq!(data(&text)["undone"][0]["new"], json!(0.25));
    let (text, _) = s.tool("plan_undo", json!({ "plan": "p1", "all": true }));
    assert_eq!(data(&text)["changesAgainstFile"], 0);
    // Take over the variant «all as lump sum» of an experiment (the example plan takes half).
    let (text, error) = s.tool("evaluate", json!({ "plan": "p1", "experiment": small_experiment() }));
    assert!(!error, "{text}");
    let result = data(&text)["result"].as_str().unwrap().to_string();
    let (text, error) = s.tool("plan_apply", json!({ "result": result, "rank": 3 }));
    assert!(!error, "{text}");
    let applied = data(&text);
    let paths: Vec<&str> = applied["changes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["path"].as_str().unwrap())
        .collect();
    assert!(
        paths.contains(&"pension.pk.lumpsum") && paths.contains(&"returns.source"),
        "{paths:?}"
    );
    // Rule 1: a new file, the opened one untouched; answers name only the file.
    let (text, error) = s.tool("plan_save", json!({ "plan": "p1" }));
    assert!(!error, "{text}");
    assert_eq!(data(&text)["saved"], "plan (AI).vplan");
    assert!(!text.contains(tmp.path().to_str().unwrap()));
    assert_eq!(std::fs::read(&file).unwrap(), original);
    let new = engine::api::read_plan(&tmp.path().join("plan (AI).vplan")).unwrap();
    assert_eq!(new["pension"]["pk"]["lumpsum"], json!(1));
    // Rule 2: declined, nothing written; accepted, replaced with a backup.
    let (question, text, _) = s.save_answering(
        json!({ "plan": "p1", "overwrite": true }),
        json!({ "action": "decline" }),
    );
    let message = question["params"]["message"].as_str().unwrap();
    assert!(
        message.contains("plan.vplan") && message.contains("pension.pk.lumpsum"),
        "{message}"
    );
    assert_eq!(question["params"]["requestedSchema"]["required"], json!(["replace"]));
    assert_eq!(data(&text)["saved"], Value::Null);
    assert_eq!(std::fs::read(&file).unwrap(), original);
    let (_, text, error) = s.save_answering(
        json!({ "plan": "p1", "overwrite": true }),
        json!({ "action": "accept", "content": { "replace": true } }),
    );
    assert!(!error, "{text}");
    assert_eq!(data(&text)["backup"], "plan.vplan.bak");
    assert_eq!(std::fs::read(tmp.path().join("plan.vplan.bak")).unwrap(), original);
    assert_eq!(
        engine::api::read_plan(&file).unwrap()["pension"]["pk"]["lumpsum"],
        json!(1)
    );
    let (text, _) = s.tool("plan_changes", json!({ "plan": "p1" }));
    assert_eq!(data(&text)["count"], 0);
    s.close();
}

/// #212/AK2, AK3: A program without questions gets no replacement (rule 2); --read-only lists no plan_save and refuses
/// the call; --allow-dir releases a further folder (rule 6).
// slow: starts the program over stdio
#[test]
fn issue_212_ak2_ak3_read_only_and_released_folders_over_stdio() {
    let tmp = tempfile::tempdir().unwrap();
    let (plans, out) = (tmp.path().join("plans"), tmp.path().join("out"));
    std::fs::create_dir(&plans).unwrap();
    std::fs::create_dir(&out).unwrap();
    let (file, _) = plan_file(&plans, "plan.vplan", "bootstrap", 300, json!([]));
    let original = std::fs::read(&file).unwrap();
    // #263: with --allow-dir plans are opened only in released folders, so the folder of the plan is released too.
    let mut s = Server::start(
        tmp.path(),
        &[
            "--allow-dir",
            plans.to_str().unwrap(),
            "--allow-dir",
            out.to_str().unwrap(),
        ],
    );
    s.initialize();
    s.tool("plan_open", json!({ "path": file.display().to_string() }));
    let (text, error) = s.tool("plan_save", json!({ "plan": "p1", "overwrite": true }));
    assert!(error && text.contains("rule 1"), "{text}");
    assert_eq!(std::fs::read(&file).unwrap(), original);
    let (text, error) = s.tool(
        "plan_save",
        json!({ "plan": "p1", "path": out.join("copy.vplan").display().to_string() }),
    );
    assert!(!error, "{text}");
    assert!(out.join("copy.vplan").exists());
    s.close();

    let mut s = Server::start(tmp.path(), &["--read-only"]);
    s.initialize();
    let tools = s.request("tools/list", json!({}))["result"]["tools"].clone();
    let names: Vec<&str> = tools
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(!names.contains(&"plan_save"), "{names:?}");
    assert!(names.contains(&"plan_set"));
    s.tool("plan_open", json!({ "path": file.display().to_string() }));
    let a = s.request(
        "tools/call",
        json!({ "name": "plan_save", "arguments": { "plan": "p1" } }),
    );
    assert_eq!(a["error"]["code"], -32602, "{a}");
    assert!(!plans.join("plan (AI).vplan").exists());
    s.close();
}

/// #263/AK2, AK3: With --allow-dir the server opens plans only inside the plan folder (also after resolving `..` and
/// symbolic links) and names only the file in its error; inside, opening, changing, computing, taking over a variant
/// and saving a new file work as without it.
// slow: starts the program and computes over stdio
#[test]
fn issue_263_ak2_ak3_plan_folder_over_stdio() {
    let tmp = tempfile::tempdir().unwrap();
    let (a, b) = (tmp.path().join("A"), tmp.path().join("B"));
    std::fs::create_dir(&a).unwrap();
    std::fs::create_dir(&b).unwrap();
    let (inside, _) = plan_file(&a, "plan.vplan", "bootstrap", 300, json!([]));
    let (outside, _) = plan_file(&b, "other.vplan", "bootstrap", 300, json!([]));
    let mut s = Server::start(tmp.path(), &["--allow-dir", a.to_str().unwrap()]);
    s.initialize();
    let mut refused = vec![outside.clone(), a.join("..").join("B").join("other.vplan")];
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&outside, a.join("link.vplan")).unwrap();
        std::os::unix::fs::symlink(&b, a.join("linked")).unwrap();
        refused.push(a.join("link.vplan"));
        refused.push(a.join("linked").join("other.vplan"));
    }
    for path in &refused {
        let (text, error) = s.tool("plan_open", json!({ "path": path.to_str().unwrap() }));
        assert!(error, "{}: {text}", path.display());
        assert!(text.contains("outside the plan folder"), "{text}");
        assert!(
            !text.contains(tmp.path().to_str().unwrap()),
            "names only the file: {text}"
        );
    }
    // Without --allow-dir as before: every path.
    let mut free = Server::start(tmp.path(), &[]);
    free.initialize();
    let (text, error) = free.tool("plan_open", json!({ "path": outside.to_str().unwrap() }));
    assert!(!error, "{text}");
    free.close();

    let (text, error) = s.tool("plan_open", json!({ "path": inside.to_str().unwrap() }));
    assert!(!error, "{text}");
    assert_eq!(data(&text)["plan"], "p1");
    let (text, error) = s.tool(
        "plan_set",
        json!({ "plan": "p1", "changes": [{ "path": "wealth.stockShare", "value": 0.3 }] }),
    );
    assert!(!error, "{text}");
    let (text, error) = s.tool("simulation_run", json!({ "plan": "p1", "paths": 200 }));
    assert!(!error, "{text}");
    let (text, error) = s.tool("result_get", json!({ "result": "r1", "view": "bands" }));
    assert!(!error, "{text}");
    let (text, error) = s.tool("evaluate", json!({ "plan": "p1", "experiment": small_experiment() }));
    assert!(!error, "{text}");
    assert_eq!(data(&text)["result"], "r2");
    let (text, error) = s.tool("plan_apply", json!({ "result": "r2", "rank": 2 }));
    assert!(!error, "{text}");
    let (text, error) = s.tool("plan_save", json!({ "plan": "p1" }));
    assert!(!error, "{text}");
    let saved = data(&text)["saved"].as_str().unwrap().to_string();
    assert!(a.join(&saved).is_file(), "{saved}");
    let (text, error) = s.tool(
        "plan_save",
        json!({ "plan": "p1", "path": b.join("copy.vplan").to_str().unwrap() }),
    );
    assert!(error && text.contains("rule 6"), "{text}");
    assert!(!b.join("copy.vplan").exists());
    s.close();
}

/// #263/AK5: The switch «read only» of the package comes as --read-only=true or --read-only=false: with true the
/// server offers no plan_save, changing and computing go on; another value is refused at the start.
// slow: starts the program and computes over stdio
#[test]
fn issue_263_ak5_read_only_switch_over_stdio() {
    let tmp = tempfile::tempdir().unwrap();
    let (file, _) = plan_file(tmp.path(), "plan.vplan", "bootstrap", 300, json!([]));
    let dir = tmp.path().to_str().unwrap();
    let names = |s: &mut Server| -> Vec<String> {
        s.request("tools/list", json!({}))["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap().to_string())
            .collect()
    };
    let mut s = Server::start(tmp.path(), &["--allow-dir", dir, "--read-only=true"]);
    s.initialize();
    let list = names(&mut s);
    assert!(!list.contains(&"plan_save".to_string()), "{list:?}");
    s.tool("plan_open", json!({ "path": file.to_str().unwrap() }));
    let (text, error) = s.tool(
        "plan_set",
        json!({ "plan": "p1", "changes": [{ "path": "wealth.stockShare", "value": 0.3 }] }),
    );
    assert!(!error, "{text}");
    let (text, error) = s.tool("simulation_run", json!({ "plan": "p1", "paths": 200 }));
    assert!(!error, "{text}");
    s.close();

    let mut s = Server::start(tmp.path(), &["--allow-dir", dir, "--read-only=false"]);
    s.initialize();
    assert!(names(&mut s).contains(&"plan_save".to_string()));
    s.close();

    let out = Command::new(env!("CARGO_BIN_EXE_vivaplan"))
        .args(["mcp", "--read-only=maybe"])
        .env("HOME", tmp.path())
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--read-only=maybe"));
}

/// #263/AK2: --allow-dir without a folder, or with a folder that does not exist, ends the start with a message instead of
/// running without the limit.
#[test]
fn issue_263_ak2_allow_dir_needs_an_existing_folder() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("none");
    for (args, want) in [
        (vec!["mcp", "--allow-dir"], "Unknown option --allow-dir"),
        (vec!["mcp", "--names", "--allow-dir"], "Unknown option --allow-dir"),
        (
            vec!["mcp", "--allow-dir", missing.to_str().unwrap()],
            "Folder not found",
        ),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_vivaplan"))
            .args(&args)
            .env("HOME", tmp.path())
            .stdin(Stdio::null())
            .output()
            .unwrap();
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(!out.status.success(), "{args:?}");
        assert!(err.contains(want), "{args:?}: {err}");
        assert!(!err.contains("panicked"), "{args:?}: {err}");
    }
}

/// #269/AK2, AK4: Over stdio a new plan opens empty with the missing inputs; after plan_set it is saved as a new file in
/// the released folder (relative path), nowhere else.
// slow: starts the program over stdio
#[test]
fn issue_269_ak2_ak4_new_plan_over_stdio() {
    let tmp = tempfile::tempdir().unwrap();
    let plans = tmp.path().join("plans");
    std::fs::create_dir(&plans).unwrap();
    let mut s = Server::start(tmp.path(), &["--allow-dir", plans.to_str().unwrap()]);
    s.initialize();
    let (text, error) = s.tool("plan_open", json!({ "new": true }));
    assert!(!error, "{text}");
    let open = data(&text);
    assert_eq!(open["file"], "(new plan)");
    assert!(open["missing"].as_array().unwrap().len() >= 9, "{open}");
    let (text, error) = s.tool(
        "plan_set",
        json!({ "plan": "p1", "changes": [
            { "path": "household.persons.0.name", "value": "Person 1" },
            { "path": "wealth.cash", "value": 50000 },
        ] }),
    );
    assert!(!error, "{text}");
    let (text, error) = s.tool("plan_save", json!({ "plan": "p1", "path": "New.vplan" }));
    assert!(!error, "{text}");
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(plans.join("New.vplan")).unwrap()).unwrap();
    assert_eq!(saved["wealth"]["cash"], 50000);
    assert!(saved.get("partner").is_none());
    let (text, error) = s.tool("plan_open", json!({ "new": true, "example": true }));
    assert!(error && text.contains("without path and example"), "{text}");
    s.close();
}
