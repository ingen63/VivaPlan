//! #157/AK2: Die Content-Security-Policy der Desktop-App erlaubt keine Verbindungen nach aussen.
//!
//! `connect-src` regelt fetch, XHR und WebSocket der Oberfläche. Erlaubt sind nur die eigene Herkunft und die
//! IPC-Quellen von Tauri (`ipc:` auf macOS/Linux, `http://ipc.localhost` auf Windows).

use std::path::Path;

const ERLAUBT: [&str; 3] = ["'self'", "ipc:", "http://ipc.localhost"];

/// Quellen in einer `connect-src`-Angabe, die nicht zu den erlaubten gehören.
fn fremde_quellen(connect_src: &str) -> Vec<&str> {
    connect_src
        .split_whitespace()
        .filter(|q| !ERLAUBT.contains(q))
        .collect()
}

fn connect_src_aus_konfiguration() -> String {
    let pfad = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let text = std::fs::read_to_string(&pfad).expect("tauri.conf.json lesbar");
    let conf: serde_json::Value = serde_json::from_str(&text).expect("tauri.conf.json ist JSON");
    conf["app"]["security"]["csp"]["connect-src"]
        .as_str()
        .expect("app.security.csp ist ein Objekt mit connect-src")
        .to_owned()
}

#[test]
fn issue_157_ak2_csp_connect_src_has_no_external_sources() {
    let connect_src = connect_src_aus_konfiguration();
    assert!(!connect_src.trim().is_empty(), "connect-src darf nicht leer sein");
    assert_eq!(
        fremde_quellen(&connect_src),
        Vec::<&str>::new(),
        "connect-src erlaubt externe Quellen: {connect_src}"
    );
}

#[test]
fn issue_157_ak2_external_sources_are_detected() {
    assert!(fremde_quellen("'self' ipc: http://ipc.localhost").is_empty());
    for fremd in [
        "https://example.com",
        "http://localhost:8080",
        "https:",
        "*",
        "wss://example.com",
        "http://ipc.localhost.example.com",
        "data:",
    ] {
        assert_eq!(
            fremde_quellen(&format!("'self' ipc: {fremd}")),
            vec![fremd],
            "{fremd} muss als fremd gelten"
        );
    }
}
