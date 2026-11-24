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
