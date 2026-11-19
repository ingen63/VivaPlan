//! #157/AK2: The content security policy of the desktop app allows no outbound connections.
//!
//! `connect-src` governs fetch, XHR and WebSocket of the user interface. Only the own origin and the
//! IPC sources of Tauri are allowed (`ipc:` on macOS/Linux, `http://ipc.localhost` on Windows).

use std::path::Path;

const ALLOWED: [&str; 3] = ["'self'", "ipc:", "http://ipc.localhost"];

/// Sources in a `connect-src` directive that are not among the allowed ones.
fn external_sources(connect_src: &str) -> Vec<&str> {
    connect_src
        .split_whitespace()
        .filter(|q| !ALLOWED.contains(q))
        .collect()
}

fn connect_src_from_config() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tauri.conf.json");
    let text = std::fs::read_to_string(&path).expect("tauri.conf.json lesbar");
    let conf: serde_json::Value = serde_json::from_str(&text).expect("tauri.conf.json ist JSON");
    conf["app"]["security"]["csp"]["connect-src"]
        .as_str()
        .expect("app.security.csp ist ein Objekt mit connect-src")
        .to_owned()
}

#[test]
fn issue_157_ak2_csp_connect_src_has_no_external_sources() {
    let connect_src = connect_src_from_config();
    assert!(!connect_src.trim().is_empty(), "connect-src darf nicht leer sein");
    assert_eq!(
        external_sources(&connect_src),
        Vec::<&str>::new(),
        "connect-src erlaubt externe Quellen: {connect_src}"
    );
}

#[test]
fn issue_157_ak2_external_sources_are_detected() {
    assert!(external_sources("'self' ipc: http://ipc.localhost").is_empty());
    for external in [
        "https://example.com",
        "http://localhost:8080",
        "https:",
        "*",
        "wss://example.com",
        "http://ipc.localhost.example.com",
        "data:",
    ] {
        assert_eq!(
            external_sources(&format!("'self' ipc: {external}")),
            vec![external],
            "{external} must count as external"
        );
    }
}
