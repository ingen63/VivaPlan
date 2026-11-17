//! Schnittstellentests: alle Befehle der Kommandozeile und der lokale Entwicklungsserver.

// #116: Speichergrenze je Testprozess (engine::test_alloc).
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

/// Beispielplanung mit angepassten Renditen als Datei.
fn plan_file(dir: &Path, source: &str, extra: impl Fn(&mut serde_json::Value)) -> String {
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root().join("crates/engine/assets/beispiel.vplan")).unwrap())
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
    assert!(text(&o).contains("Verwendung"));
    // #61: Kurzfassung des Haftungsausschlusses in einer Zeile
    assert!(text(&o).contains("Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung."));
    let o = run(&["schema"]);
    assert!(o.status.success());
    let schema: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(schema["properties"]["returns"].is_object());
}

// #93/AK2: `vivaplan --version` nennt die Version des Arbeitsbereichs (Cargo.toml, `version = "…"` unter `[workspace.package]`).
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
        t.contains("Vermögen am Ende") && t.contains("Ereignisse:") && t.contains("(Alter "),
        "{t}"
    );
    assert!(std::fs::read_to_string(&csv).unwrap().lines().count() > 300);
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
            assert!(t.contains("Läufe") && t.contains("Perzentil"), "{src}: {t}");
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
    assert!(text(&o).contains("gültig"));
    // Ohne Zeitreihe und ohne Anlagestrategie (#69) fehlen die Renditen für «rolling».
    let bad = plan_file(&d, "rolling", |v| {
        v["returns"]["series"] = serde_json::Value::Null;
        v["returns"].as_object_mut().unwrap().remove("strategy");
    });
    let o = run(&["validate", &bad]);
    assert!(!o.status.success());
    assert!(text(&o).contains("FEHLER"));
    let o = run(&["run", "gibt-es-nicht.vplan"]);
    assert!(!o.status.success());
    assert!(text(&o).contains("Fehler"));
    let o = run(&["validate"]);
    assert!(text(&o).contains("Projektdatei fehlt"));
}

#[test]
fn bench_runs_paths() {
    let d = tmp("bench");
    let p = plan_file(&d, "fixed", |_| {});
    let o = run(&["bench", &p, "--paths", "200"]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("200 Läufe"));
}

#[test]
fn import_legacy_config() {
    let d = tmp("legacy");
    let out = d.join("alt.vplan");
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
    // #94: die Originaltabelle liegt nur lokal; ohne sie entfällt dieser Teil.
    let shiller = root().join("crates/engine/data/quellen/shiller-index.ods");
    if shiller.exists() {
        let o = run(&["import-shiller", shiller.to_str().unwrap(), "--library", l]);
        assert!(o.status.success(), "{}", text(&o));
        assert!(
            text(&o).contains("Geschrieben: Shiller CHF + CH-Obligationen.csv"),
            "{}",
            text(&o)
        );
    } else {
        eprintln!("übersprungen: {} fehlt (#94)", shiller.display());
    }
    let cube = root().join("crates/engine/data/quellen/snb-zimoma.csv");
    let o = run(&["import-snb", cube.to_str().unwrap(), "--library", l]);
    assert!(o.status.success(), "{}", text(&o));
    assert!(text(&o).contains("zimoma übernommen"));
    let o = run(&["import-snb", "--library", l]);
    assert!(text(&o).contains("SNB-Datei fehlt"));
    let o = run(&["import-shiller", "--library", l]);
    assert!(text(&o).contains("Shiller-Datei fehlt"));
}

// ---------- Entwicklungsserver ----------

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
    // #83: Warten auf den Zustand (Port offen) statt fester Wartezeit; bis 30 s, damit auch ein langsamer Runner
    // unter Abdeckungsmessung den Server starten kann.
    for _ in 0..600 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return srv;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    panic!("Entwicklungsserver startet nicht");
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
    assert!(h.contains("400") && b.contains("Pfad fehlt"));
    let (_, b) = http(p, "POST", "/api/library_list", "{}");
    assert!(b.contains("Shiller USD.csv"));
    let (h, b) = http(p, "POST", "/api/open_plan", "{}");
    assert!(h.contains("400") && b.contains("Desktop-App"));
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
        t.contains("Ausgangslage") && t.contains("Fixe Rate") && t.contains("Guardrails"),
        "{t}"
    );
    let r: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    assert_eq!(r.as_array().unwrap().len(), 4);
    for s in r.as_array().unwrap() {
        assert!(s["validated"]["ruin"].as_f64().unwrap() <= 1.0);
        assert!(s["validated"]["median"].as_f64().unwrap() >= 0.0);
    }
    assert!(!run(&["optimize", &f, "--strategy", "unsinn"]).status.success());
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
    assert!(t.contains("Ruin ≤ 2.0 % bis Alter 90"), "{t}");
    assert!(t.contains("Fixe Rate") && !t.contains("Guardrails"), "{t}");
    let t = text(&run(&["optimize", &f, "--max-ruin", "8", "--strategy", "vpw"]));
    assert!(
        t.contains("Ruin ≤ 8.0 %") && t.contains("VPW") && !t.contains("Fixe Rate"),
        "{t}"
    );
}

#[test]
fn optimize_goal_two_needs_a_property_and_is_selectable() {
    let d = tmp("optimize-goal");
    let settings = serde_json::json!({ "paths": 100, "checkPaths": 100, "samples": 1, "strategies": ["fixed_rate"] });
    let f = plan_file(&d, "parametric", |p| p["optimization"] = settings.clone());
    // Ziel 2 mit Immobilie der Beispielplanung
    let t = text(&run(&["optimize", &f, "--goal", "2", "--min-consumption", "50"]));
    assert!(
        t.contains("Ziel 2 «Kein Notverkauf von Immobilien»") && t.contains("ungeplanter Immobilienverkauf"),
        "{t}"
    );
    // Ungültiges Ziel
    assert!(!run(&["optimize", &f, "--goal", "9"]).status.success());
    // Ziel 1 ausdrücklich gewählt: wie bisher
    let t = text(&run(&["optimize", &f, "--goal", "1"]));
    assert!(t.contains("Ziel: Ruin"), "{t}");
    // Ohne Immobilie: Fehlermeldung
    let g = plan_file(&d, "bootstrap", |p| {
        p["optimization"] = settings.clone();
        p["realestate"]["properties"] = serde_json::json!([]);
    });
    assert!(!run(&["optimize", &g, "--goal", "2"]).status.success());
}
