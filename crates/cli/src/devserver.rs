//! Local development server: serves the user interface from `crates/app/ui` and answers `/api/<command>`
//! like the desktop app. Listens on 127.0.0.1 only; there are no file dialogs here.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

pub fn serve(port: u16, ui: PathBuf, library: PathBuf) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!(
        "User interface: http://127.0.0.1:{port}/  (folder {}, data library {})",
        ui.display(),
        library.display()
    );
    // One request per thread: during a long calculation the user interface must be able to
    // poll the progress and send the cancellation.
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let (ui, library) = (ui.clone(), library.clone());
        std::thread::spawn(move || {
            if let Err(e) = handle(stream, &ui, &library) {
                eprintln!("Request failed: {e}");
            }
        });
    }
    Ok(())
}

fn handle(mut stream: TcpStream, ui: &Path, library: &Path) -> std::io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or("/").to_string());
    let mut length = 0usize;
    loop {
        let mut h = String::new();
        if reader.read_line(&mut h)? == 0 || h == "\r\n" {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            if k.eq_ignore_ascii_case("content-length") {
                length = v.trim().parse().unwrap_or(0);
            }
        }
    }
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;

    let (status, ctype, out): (&str, &str, Vec<u8>) = if method == "POST" && target.starts_with("/api/") {
        let args: Value = serde_json::from_slice(&body).unwrap_or(json!({}));
        match call(&target[5..], &args, library) {
            Ok(v) => ("200 OK", "application/json", v.to_string().into_bytes()),
            Err(e) => (
                "400 Bad Request",
                "application/json",
                error_body(&e).to_string().into_bytes(),
            ),
        }
    } else {
        let rel = target.split('?').next().unwrap_or("/").trim_start_matches('/');
        let rel = if rel.is_empty() { "index.html" } else { rel };
        let file = ui.join(rel);
        match (rel.contains(".."), std::fs::read(&file)) {
            (false, Ok(data)) => ("200 OK", content_type(&file), data),
            _ => ("404 Not Found", "text/plain", b"not found".to_vec()),
        }
    };
    write!(stream, "HTTP/1.1 {status}\r\nContent-Type: {ctype}; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", out.len())?;
    stream.write_all(&out)
}

fn content_type(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html",
        Some("js") => "text/javascript",
        Some("css") => "text/css",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("md") => "text/markdown",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

/// #171: Answer to a failed command: German text, code and parameters of the message (the user interface translates it).
fn error_body(e: &engine::message::Message) -> Value {
    json!({ "error": e.text(), "code": e.code(), "params": e.params() })
}

/// #142: the calculation commands are answered by the same dispatcher that the desktop app's contract test checks
/// (`engine::dispatch`); without a plan file, relative paths start in the working directory.
fn call(cmd: &str, args: &Value, library: &Path) -> Result<Value, engine::message::Message> {
    engine::dispatch::call(cmd, args, library, &std::env::current_dir().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    //! #105/AK1: the dev server without a browser. `call` is the command dispatcher; `serve` and `handle` run on 127.0.0.1.
    use super::*;
    use std::net::TcpStream;
    use std::time::Duration;

    /// Separate data library per call (`library_list` creates the shipped series there); deleted when dropped.
    fn library() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("vivaplan-devserver-lib-")
            .tempdir()
            .unwrap()
    }

    /// Temporary folder that is deleted when the returned guard is dropped (end of the test).
    fn tmp(name: &str) -> (tempfile::TempDir, PathBuf) {
        let d = tempfile::Builder::new()
            .prefix(&format!("vivaplan-devserver-{name}-"))
            .tempdir()
            .unwrap();
        let p = d.path().to_path_buf();
        (d, p)
    }

    /// Small, fast plan: fixed returns, no scenarios.
    fn plan() -> Value {
        let mut p = engine::api::example_plan();
        p["returns"]["source"] = json!("fixed");
        p["scenarios"]["sweeps"] = json!([]);
        p
    }

    fn call_ok(cmd: &str, args: Value) -> Value {
        call(cmd, &args, library().path()).unwrap_or_else(|e| panic!("{cmd}: {e}"))
    }

    #[test]
    fn commands_without_a_plan() {
        assert!(call_ok("app_info", json!({}))["version"].is_string());
        assert!(call_ok("progress_get", json!({}))["active"].is_boolean());
        assert!(call_ok("cancel_job", json!({})).is_null());
        assert!(call_ok("example_plan", json!({}))["household"].is_object());
        assert!(call_ok("library_list", json!({}))["entries"].is_array());
        assert!(call_ok("opt_params", json!({ "plan": plan() })).is_array());
    }

    #[test]
    fn commands_with_a_plan() {
        let args = json!({ "plan": plan() });
        assert!(call_ok("validate", args.clone()).is_array());
        assert!(call_ok("upgrade_plan", args.clone()).is_object());
        assert!(call_ok("preview", args.clone()).is_object());
        let run = call_ok("run_simulation", args.clone());
        assert!(run["single"]["summary"]["endTotal"].is_number(), "{run}");
        assert!(call_ok("bench_plan", args.clone()).is_object());
        let re = call_ok("rerun_paths", json!({ "plan": plan(), "picks": [] }));
        assert!(re.is_object());
        let o = call_ok("optimize", json!({ "plan": plan(), "request": {} }));
        assert!(o["outcomes"].is_array());
    }

    #[test]
    fn progress_and_cancel_follow_the_job_id() {
        // #107/AK2, #107/AK4: the user interface passes the id at the start and uses it to poll progress and cancel.
        let before = call_ok("progress_get", json!({ "jobId": 424242 }));
        assert_eq!(
            (before["id"].as_u64(), before["active"].as_bool()),
            (Some(424242), Some(false))
        );
        assert!(call_ok("cancel_job", json!({ "jobId": 424242 })).is_null());
        let run = call_ok("run_simulation", json!({ "plan": plan(), "jobId": 424242 }));
        assert!(run["error"].is_null(), "{run}");
        let after = call_ok("progress_get", json!({ "jobId": 424242 }));
        assert_eq!(
            (after["active"].as_bool(), after["cancelled"].as_bool()),
            (Some(false), Some(false))
        );
        // the job with this id has calculated
        assert!(after["total"].as_u64().unwrap() >= 1, "{after}");
        // without an id: the most recently started calculation; cancelling without an id hits all running ones
        assert_eq!(call_ok("progress_get", json!({}))["id"], json!(424242));
        assert!(call_ok("cancel_job", json!({})).is_null());
        let o = call_ok("optimize", json!({ "plan": plan(), "request": {}, "jobId": 424243 }));
        assert!(o["outcomes"].is_array());
        assert_eq!(call_ok("progress_get", json!({ "jobId": 424243 }))["id"], json!(424243));
    }

    #[test]
    fn invalid_arguments_and_unknown_commands_are_errors() {
        let lib = library();
        let err = |cmd: &str, args: Value| call(cmd, &args, lib.path()).unwrap_err();
        assert!(err("rerun_paths", json!({ "plan": plan(), "picks": "x" })).contains("invalid type"));
        assert_eq!(err("series_info", json!({})), "Path missing");
        // #171: the answer carries text, code and parameters as an object
        assert_eq!(
            super::error_body(&err("open_dialog", json!({}))),
            json!({ "error": "Command “open_dialog” is only available in the desktop app", "code": "command_desktop_only", "params": { "cmd": "open_dialog" } })
        );
        assert!(!err("series_info", json!({ "path": "/does/not/exist.csv" })).is_empty());
        assert!(!err("optimize", json!({ "plan": plan(), "request": "nonsense" })).is_empty());
        assert_eq!(
            err("open_dialog", json!({})),
            "Command “open_dialog” is only available in the desktop app"
        );
    }

    #[test]
    fn series_info_passes_the_file_to_the_parser() {
        let (_tmp, dir) = tmp("series");
        std::fs::write(dir.join("s.csv"), "Date,Value\n2000-01,1.0\n2000-02,1.1\n2000-03,1.2\n").unwrap();
        let e = call(
            "series_info",
            &json!({ "path": dir.join("s.csv").to_string_lossy() }),
            library().path(),
        )
        .unwrap_err();
        assert!(e.contains("column “Stocks” missing"), "{e}");
    }

    /// #142: without a plan file, relative paths start in the working directory (in the test the folder `crates/cli`).
    #[test]
    fn relative_paths_without_a_plan_file_start_in_the_working_directory() {
        let rel = "../engine/data/Shiller CHF.csv";
        assert!(std::env::current_dir().unwrap().join(rel).is_file());
        assert!(call(
            "series_info",
            &json!({ "path": rel, "planPath": null }),
            library().path()
        )
        .is_ok());
        assert!(call(
            "series_info",
            &json!({ "path": rel, "planPath": "/does/not/exist/p.vplan" }),
            library().path()
        )
        .is_err());
    }

    #[test]
    fn content_types_follow_the_extension() {
        for (name, t) in [
            ("a.html", "text/html"),
            ("a.js", "text/javascript"),
            ("a.css", "text/css"),
            ("a.svg", "image/svg+xml"),
            ("a.png", "image/png"),
            ("a.md", "text/markdown"),
            ("a.json", "application/json"),
            ("a.bin", "application/octet-stream"),
            ("a", "application/octet-stream"),
        ] {
            assert_eq!(content_type(Path::new(name)), t, "{name}");
        }
    }

    /// One request to the running server; returns the status line and the body.
    fn request(port: u16, head: &str, body: &str) -> (String, String) {
        let mut s = (0..100)
            .find_map(|_| {
                TcpStream::connect(("127.0.0.1", port)).ok().or_else(|| {
                    std::thread::sleep(Duration::from_millis(50));
                    None
                })
            })
            .expect("server reachable");
        write!(s, "{head}\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        let (h, b) = out.split_once("\r\n\r\n").unwrap();
        (h.lines().next().unwrap().to_string(), b.to_string())
    }

    #[test]
    fn serves_files_and_the_api_and_refuses_paths_outside() {
        let (_ui, ui) = tmp("ui");
        std::fs::write(ui.join("index.html"), "<p>Start</p>").unwrap();
        std::fs::write(ui.join("app.js"), "1").unwrap();
        let port = {
            let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            l.local_addr().unwrap().port()
        };
        let lib = library();
        let lib_path = lib.path().to_path_buf();
        std::thread::spawn(move || serve(port, ui, lib_path));
        assert_eq!(
            request(port, "GET / HTTP/1.1", ""),
            ("HTTP/1.1 200 OK".into(), "<p>Start</p>".into())
        );
        assert_eq!(request(port, "GET /app.js?x=1 HTTP/1.1", "").1, "1");
        assert_eq!(
            request(port, "GET /does-not-exist HTTP/1.1", "").0,
            "HTTP/1.1 404 Not Found"
        );
        assert_eq!(
            request(port, "GET /../Cargo.toml HTTP/1.1", "").0,
            "HTTP/1.1 404 Not Found"
        );
        let (st, body) = request(port, "POST /api/example_plan HTTP/1.1", "{}");
        assert_eq!(st, "HTTP/1.1 200 OK");
        assert!(serde_json::from_str::<Value>(&body).unwrap()["household"].is_object());
        let (st, body) = request(port, "POST /api/does_not_exist HTTP/1.1", "no json");
        assert_eq!(st, "HTTP/1.1 400 Bad Request");
        assert!(body.contains("is only available in the desktop app"), "{body}");
        // GET on /api/ is not a request to the dispatcher
        assert_eq!(
            request(port, "GET /api/example_plan HTTP/1.1", "").0,
            "HTTP/1.1 404 Not Found"
        );
    }

    #[test]
    fn serve_fails_when_the_port_is_taken() {
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(serve(port, PathBuf::from("."), library().path().to_path_buf()).is_err());
    }
}
