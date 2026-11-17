//! Lokaler Entwicklungsserver: liefert die Oberfläche aus `crates/app/ui` und beantwortet `/api/<befehl>`
//! wie die Desktop-App. Lauscht nur auf 127.0.0.1; Dateidialoge gibt es hier nicht.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};

use engine::api;
use serde_json::{json, Value};

pub fn serve(port: u16, ui: PathBuf, library: PathBuf) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    println!(
        "Oberfläche: http://127.0.0.1:{port}/  (Ordner {}, Datenbibliothek {})",
        ui.display(),
        library.display()
    );
    // Eine Anfrage pro Thread: Während einer langen Berechnung muss die Oberfläche den
    // Fortschritt abfragen und den Abbruch senden können.
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let (ui, library) = (ui.clone(), library.clone());
        std::thread::spawn(move || {
            if let Err(e) = handle(stream, &ui, &library) {
                eprintln!("Anfrage fehlgeschlagen: {e}");
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
                json!({ "error": e }).to_string().into_bytes(),
            ),
        }
    } else {
        let rel = target.split('?').next().unwrap_or("/").trim_start_matches('/');
        let rel = if rel.is_empty() { "index.html" } else { rel };
        let file = ui.join(rel);
        match (rel.contains(".."), std::fs::read(&file)) {
            (false, Ok(data)) => ("200 OK", content_type(&file), data),
            _ => ("404 Not Found", "text/plain", b"nicht gefunden".to_vec()),
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
        _ => "application/octet-stream",
    }
}

fn base_dir(args: &Value) -> PathBuf {
    args.get("path")
        .and_then(Value::as_str)
        .and_then(|p| Path::new(p).parent().map(Path::to_path_buf))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

/// Kennung der Berechnung (`jobId`), die die Oberfläche beim Start mitgibt.
fn job_id(args: &Value) -> Option<u64> {
    args.get("jobId").and_then(Value::as_u64)
}

fn call(cmd: &str, args: &Value, library: &Path) -> Result<Value, String> {
    let plan = args.get("plan").cloned().unwrap_or(Value::Null);
    let to = |v: &dyn erased::Ser| v.json();
    let lib = Some(library);
    Ok(match cmd {
        "app_info" => serde_json::json!({ "version": env!("CARGO_PKG_VERSION"), "buildEpoch": null }),
        "bench_plan" => to(&api::bench(&plan, &base_dir(args), lib)?),
        "progress_get" => to(&match job_id(args) {
            Some(id) => engine::progress::snapshot_of(id),
            None => engine::progress::snapshot(),
        }),
        "cancel_job" => {
            match job_id(args) {
                Some(id) => {
                    engine::progress::cancel_job(id);
                }
                None => engine::progress::cancel(),
            }
            Value::Null
        }
        "opt_params" => to(&api::opt_params(&plan)),
        "example_plan" => api::example_plan(),
        "validate" => to(&api::validate(&plan)),
        "upgrade_plan" => to(&api::upgrade_plan(&plan)),
        "preview" => to(&api::preview(&plan, &base_dir(args), lib)),
        "run_simulation" => to(&api::run_as(job_id(args), &plan, &base_dir(args), lib)),
        "rerun_paths" => {
            let picks: Vec<(u32, usize)> =
                serde_json::from_value(args.get("picks").cloned().unwrap_or_default()).map_err(|e| e.to_string())?;
            to(&api::rerun_paths(&plan, &picks, &base_dir(args), lib))
        }
        "series_info" => {
            let p = args.get("path").and_then(Value::as_str).ok_or("Pfad fehlt")?;
            let cwd = std::env::current_dir().unwrap_or_default();
            to(&api::series_info(p, &cwd, lib).map_err(|e| e.to_string())?)
        }
        "optimize" => {
            let mut req = args.get("request").cloned().unwrap_or(json!({}));
            if let (Some(id), Some(obj)) = (job_id(args), req.as_object_mut()) {
                obj.insert("jobId".into(), id.into());
            }
            to(&api::optimize(&plan, &req, &base_dir(args), lib).map_err(|e| e.to_string())?)
        }
        "library_list" => to(&api::library_list(library).map_err(|e| e.to_string())?),
        _ => return Err(format!("Befehl «{cmd}» gibt es nur in der Desktop-App")),
    })
}

/// Kleine Hilfe, damit verschiedene Ergebnis-Typen einheitlich zu JSON werden.
mod erased {
    pub trait Ser {
        fn json(&self) -> serde_json::Value;
    }
    impl<T: serde::Serialize> Ser for T {
        fn json(&self) -> serde_json::Value {
            serde_json::to_value(self).expect("serialisierbar")
        }
    }
}

#[cfg(test)]
mod tests {
    //! #105/AK1: Der Dev-Server ohne Browser. `call` ist der Befehlsverteiler; `serve` und `handle` laufen auf 127.0.0.1.
    use super::*;
    use std::net::TcpStream;
    use std::time::Duration;

    /// Eigene Datenbibliothek je Test (`library_list` legt dort die ausgelieferten Zeitreihen an).
    fn library() -> PathBuf {
        tmp(&format!("lib-{:?}", std::thread::current().id()).replace(['(', ')'], ""))
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vivaplan-devserver-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// Kleine, schnelle Planung: feste Renditen, keine Szenarien.
    fn plan() -> Value {
        let mut p = api::example_plan();
        p["returns"]["source"] = json!("fixed");
        p["scenarios"]["sweeps"] = json!([]);
        p
    }

    fn call_ok(cmd: &str, args: Value) -> Value {
        call(cmd, &args, &library()).unwrap_or_else(|e| panic!("{cmd}: {e}"))
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
        // #107/AK2, #107/AK4: Die Oberfläche gibt die Kennung beim Start mit und fragt damit Stand und Abbruch ab.
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
        assert!(after["total"].as_u64().unwrap() >= 1, "{after}"); // der Job dieser Kennung hat gerechnet
                                                                   // ohne Kennung: die zuletzt gestartete Berechnung; Abbruch ohne Kennung trifft alle laufenden
        assert_eq!(call_ok("progress_get", json!({}))["id"], json!(424242));
        assert!(call_ok("cancel_job", json!({})).is_null());
        let o = call_ok("optimize", json!({ "plan": plan(), "request": {}, "jobId": 424243 }));
        assert!(o["outcomes"].is_array());
        assert_eq!(call_ok("progress_get", json!({ "jobId": 424243 }))["id"], json!(424243));
    }

    #[test]
    fn invalid_arguments_and_unknown_commands_are_errors() {
        let lib = library();
        let err = |cmd: &str, args: Value| call(cmd, &args, &lib).unwrap_err();
        assert!(err("rerun_paths", json!({ "plan": plan(), "picks": "x" })).contains("invalid type"));
        assert_eq!(err("series_info", json!({})), "Pfad fehlt");
        assert!(!err("series_info", json!({ "path": "/gibt/es/nicht.csv" })).is_empty());
        assert!(!err("optimize", json!({ "plan": plan(), "request": "unsinn" })).is_empty());
        assert_eq!(
            err("open_dialog", json!({})),
            "Befehl «open_dialog» gibt es nur in der Desktop-App"
        );
    }

    #[test]
    fn series_info_passes_the_file_to_the_parser() {
        let dir = tmp("series");
        std::fs::write(dir.join("s.csv"), "Date,Value\n2000-01,1.0\n2000-02,1.1\n2000-03,1.2\n").unwrap();
        let e = call(
            "series_info",
            &json!({ "path": dir.join("s.csv").to_string_lossy() }),
            &library(),
        )
        .unwrap_err();
        assert!(e.contains("Spalte «Stocks» fehlt"), "{e}");
    }

    #[test]
    fn base_dir_is_the_folder_of_the_plan_file_or_the_working_directory() {
        assert_eq!(base_dir(&json!({ "path": "/a/b/plan.vplan" })), PathBuf::from("/a/b"));
        assert_eq!(base_dir(&json!({})), std::env::current_dir().unwrap());
        assert_eq!(base_dir(&json!({ "path": 5 })), std::env::current_dir().unwrap());
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
            ("a.bin", "application/octet-stream"),
            ("a", "application/octet-stream"),
        ] {
            assert_eq!(content_type(Path::new(name)), t, "{name}");
        }
    }

    /// Eine Anfrage an den laufenden Server; gibt Statuszeile und Rumpf zurück.
    fn request(port: u16, head: &str, body: &str) -> (String, String) {
        let mut s = (0..100)
            .find_map(|_| {
                TcpStream::connect(("127.0.0.1", port)).ok().or_else(|| {
                    std::thread::sleep(Duration::from_millis(50));
                    None
                })
            })
            .expect("Server erreichbar");
        write!(s, "{head}\r\nContent-Length: {}\r\n\r\n{body}", body.len()).unwrap();
        let mut out = String::new();
        s.read_to_string(&mut out).unwrap();
        let (h, b) = out.split_once("\r\n\r\n").unwrap();
        (h.lines().next().unwrap().to_string(), b.to_string())
    }

    #[test]
    fn serves_files_and_the_api_and_refuses_paths_outside() {
        let ui = tmp("ui");
        std::fs::write(ui.join("index.html"), "<p>Start</p>").unwrap();
        std::fs::write(ui.join("app.js"), "1").unwrap();
        let port = {
            let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            l.local_addr().unwrap().port()
        };
        let lib = library();
        std::thread::spawn(move || serve(port, ui, lib));
        assert_eq!(
            request(port, "GET / HTTP/1.1", ""),
            ("HTTP/1.1 200 OK".into(), "<p>Start</p>".into())
        );
        assert_eq!(request(port, "GET /app.js?x=1 HTTP/1.1", "").1, "1");
        assert_eq!(
            request(port, "GET /gibt-es-nicht HTTP/1.1", "").0,
            "HTTP/1.1 404 Not Found"
        );
        assert_eq!(
            request(port, "GET /../Cargo.toml HTTP/1.1", "").0,
            "HTTP/1.1 404 Not Found"
        );
        let (st, body) = request(port, "POST /api/example_plan HTTP/1.1", "{}");
        assert_eq!(st, "HTTP/1.1 200 OK");
        assert!(serde_json::from_str::<Value>(&body).unwrap()["household"].is_object());
        let (st, body) = request(port, "POST /api/gibt_es_nicht HTTP/1.1", "kein json");
        assert_eq!(st, "HTTP/1.1 400 Bad Request");
        assert!(body.contains("gibt es nur in der Desktop-App"), "{body}");
        // GET auf /api/ ist keine Anfrage an den Verteiler
        assert_eq!(
            request(port, "GET /api/example_plan HTTP/1.1", "").0,
            "HTTP/1.1 404 Not Found"
        );
    }

    #[test]
    fn serve_fails_when_the_port_is_taken() {
        let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(serve(port, PathBuf::from("."), library()).is_err());
    }
}
