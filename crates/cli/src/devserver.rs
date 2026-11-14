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

fn call(cmd: &str, args: &Value, library: &Path) -> Result<Value, String> {
    let plan = args.get("plan").cloned().unwrap_or(Value::Null);
    let to = |v: &dyn erased::Ser| v.json();
    let lib = Some(library);
    Ok(match cmd {
        "app_info" => serde_json::json!({ "version": env!("CARGO_PKG_VERSION"), "buildEpoch": null }),
        "bench_plan" => to(&api::bench(&plan, &base_dir(args), lib)?),
        "progress_get" => to(&engine::progress::snapshot()),
        "cancel_job" => {
            engine::progress::cancel();
            Value::Null
        }
        "opt_params" => to(&api::opt_params(&plan)),
        "example_plan" => api::example_plan(),
        "validate" => to(&api::validate(&plan)),
        "upgrade_plan" => to(&api::upgrade_plan(&plan)),
        "preview" => to(&api::preview(&plan, &base_dir(args), lib)),
        "run_simulation" => to(&api::run(&plan, &base_dir(args), lib)),
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
            let req = args.get("request").cloned().unwrap_or(json!({}));
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
