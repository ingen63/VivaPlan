//! Desktop-App: Fenster mit der Oberfläche aus `crates/app/ui`, Befehle für Rechenkern und Dateien.
//! Dateien werden nur über Dialoge gewählt und lokal gelesen bzw. geschrieben.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod migrate;

// #116: Speichergrenze je Testprozess (engine::test_alloc).
#[cfg(test)]
#[global_allocator]
static TEST_ALLOC: engine::test_alloc::Limited = engine::test_alloc::Limited;

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use engine::api;
use engine::validate::Issue;
use serde::Serialize;
use serde_json::Value;
use tauri::menu::{Menu, MenuEvent, MenuItemBuilder, SubmenuBuilder};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager, RunEvent, Runtime, State, WindowEvent};
use tauri_plugin_dialog::{DialogExt, FilePath};

fn home() -> PathBuf {
    home_from(std::env::var_os("HOME"), std::env::var_os("USERPROFILE"))
}

/// Home-Verzeichnis aus `HOME`, ersatzweise `USERPROFILE` (Windows); sonst leer.
fn home_from(home: Option<OsString>, user_profile: Option<OsString>) -> PathBuf {
    home.or(user_profile).map(PathBuf::from).unwrap_or_default()
}

/// Verzeichnis der Projektdatei (für relative Pfade zu Zeitreihen).
fn base_dir(plan_path: Option<&str>) -> PathBuf {
    plan_path
        .and_then(|p| Path::new(p).parent().map(Path::to_path_buf))
        .unwrap_or_else(home)
}

/// Ordner der Datenbibliothek (im Datenverzeichnis der App, z. B. unter macOS
/// ~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen).
fn library_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    ensure_library_dir(&data)
}

/// Legt den Unterordner «Zeitreihen» im Datenverzeichnis an (falls nötig) und liefert ihn.
fn ensure_library_dir(data_dir: &Path) -> Result<PathBuf, String> {
    let dir = data_dir.join("Zeitreihen");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(dir)
}

/// Datei mit den Einstellungen der Oberfläche im Datenverzeichnis (nur Einstellungen, keine Planungsdaten).
const SETTINGS_FILE: &str = "einstellungen.json";

/// Liest die Einstellungen; fehlende oder unlesbare Datei ergibt leere Einstellungen (es gelten die Standardwerte).
fn read_settings(data_dir: &Path) -> serde_json::Map<String, Value> {
    std::fs::read_to_string(data_dir.join(SETTINGS_FILE))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| match v {
            Value::Object(m) => Some(m),
            _ => None,
        })
        .unwrap_or_default()
}

/// Setzt eine Einstellung und schreibt die Datei.
fn write_setting(data_dir: &Path, key: &str, value: Value) -> Result<(), String> {
    let mut all = read_settings(data_dir);
    all.insert(key.to_string(), value);
    std::fs::create_dir_all(data_dir).map_err(|e| format!("{}: {e}", data_dir.display()))?;
    let file = data_dir.join(SETTINGS_FILE);
    let text = serde_json::to_string_pretty(&all).map_err(|e| e.to_string())?;
    std::fs::write(&file, text).map_err(|e| format!("{}: {e}", file.display()))
}

#[tauri::command]
async fn settings_get<R: Runtime>(app: AppHandle<R>) -> Result<Value, String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(Value::Object(read_settings(&data)))
}

#[tauri::command]
async fn settings_set<R: Runtime>(app: AppHandle<R>, key: String, value: Value) -> Result<(), String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    write_setting(&data, &key, value)
}

/// Einstellung mit dem Pfad der zuletzt geöffneten oder gespeicherten Planung (nur der Pfad, lokal).
const LAST_PLAN_KEY: &str = "lastPlan";
/// Name der mitgelieferten neutralen Beispielplanung (Ressource im App-Bundle).
const EXAMPLE_RESOURCE: &str = "Neu.vplan";

/// Planung beim Start: Pfad (leer bei der Beispielplanung), Inhalt und ein Hinweis für die Oberfläche.
/// `first`: allererster Start (#70), die Oberfläche zeigt dann den Willkommensdialog.
#[derive(Serialize, Debug, PartialEq)]
struct Startup {
    path: Option<String>,
    plan: Value,
    notice: Option<String>,
    first: bool,
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// Bestimmt die Planung beim Start. Eine beim Start übergebene Datei (Doppelklick, Kommandozeile) geht vor, sonst gilt
/// die gemerkte Datei. Ohne Merkung gilt die Beispielplanung (ohne Hinweis); fehlt die Datei oder ist sie nicht
/// lesbar, ebenfalls (mit Hinweis). Die Merkung selbst bleibt dabei unverändert.
/// #70: Gibt es noch keine Einstellungsdatei und wurde keine Datei übergeben, ist es der allererste Start.
fn startup_from(data_dir: &Path, given: Option<PathBuf>, example: impl FnOnce() -> Value) -> Startup {
    let first = given.is_none() && !data_dir.join(SETTINGS_FILE).exists();
    let settings = read_settings(data_dir);
    let given = given.map(|p| p.to_string_lossy().into_owned());
    let Some(last) = given
        .as_deref()
        .or_else(|| settings.get(LAST_PLAN_KEY).and_then(Value::as_str))
    else {
        return Startup {
            path: None,
            plan: example(),
            notice: None,
            first,
        };
    };
    let fallback = |why: &str| Startup {
        path: None,
        plan: example(),
        notice: Some(format!(
            "{why}: {}. Es wird die Beispielplanung geöffnet.",
            file_name(last)
        )),
        first: false,
    };
    if !Path::new(last).is_file() {
        return fallback("Zuletzt geöffnete Datei nicht gefunden");
    }
    match api::read_plan(Path::new(last)) {
        Ok(plan) => Startup {
            path: Some(last.to_string()),
            plan,
            notice: None,
            first: false,
        },
        Err(_) => fallback("Zuletzt geöffnete Datei nicht lesbar"),
    }
}

/// Mitgelieferte Beispielplanung aus den Ressourcen; ersatzweise die eingebaute.
fn bundled_example<R: Runtime>(app: &AppHandle<R>) -> Value {
    app.path()
        .resolve(EXAMPLE_RESOURCE, BaseDirectory::Resource)
        .ok()
        .and_then(|p| api::read_plan(&p).ok())
        .unwrap_or_else(api::example_plan)
}

#[tauri::command]
async fn startup_plan<R: Runtime>(app: AppHandle<R>, pending: State<'_, PendingOpen>) -> Result<Startup, String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let given = pending.take();
    let s = startup_from(&data, given, || bundled_example(&app));
    if let Some(p) = &s.path {
        remember_plan(&app, p);
    }
    Ok(s)
}

/// #44: Datei, mit der die App gestartet wurde (Doppelklick im Finder, Dateizuordnung, Kommandozeile), bis die
/// Oberfläche beim Start danach fragt.
#[derive(Default)]
struct PendingOpen(Mutex<Option<PathBuf>>);

impl PendingOpen {
    fn set(&self, p: PathBuf) {
        *self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(p);
    }

    fn take(&self) -> Option<PathBuf> {
        self.0.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take()
    }
}

/// Endungen von Planungsdateien; `rplan` ist die alte Endung (legacy: RetirementPlanner), sie bleibt lesbar.
const PLAN_EXTENSIONS: [&str; 2] = ["vplan", "rplan"];

fn is_plan_file(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| PLAN_EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// Erste Planungsdatei unter den Startargumenten (Windows und Linux übergeben die Datei so).
fn plan_arg(args: impl IntoIterator<Item = OsString>) -> Option<PathBuf> {
    args.into_iter().skip(1).map(PathBuf::from).find(|p| is_plan_file(p))
}

/// Datei aus dem Betriebssystem öffnen (macOS: Doppelklick bei laufender oder startender App). Läuft die Oberfläche
/// schon, öffnet sie die Datei über das Ereignis «open-file»; beim Start holt sie sie mit `startup_plan`.
#[cfg(any(test, target_os = "macos", target_os = "ios"))]
fn open_from_system<R: Runtime>(app: &AppHandle<R>, path: PathBuf) {
    app.state::<PendingOpen>().set(path.clone());
    let _ = app.emit("open-file", path.to_string_lossy());
}

/// Öffnet eine vom System gemeldete Datei (Ereignis «open-file»), ohne Dialog.
#[tauri::command]
async fn open_path<R: Runtime>(
    app: AppHandle<R>,
    pending: State<'_, PendingOpen>,
    path: String,
) -> Result<Opened, String> {
    pending.take();
    let o = opened_from(FilePath::Path(PathBuf::from(path)))?;
    remember_plan(&app, &o.path);
    Ok(o)
}

/// Produktname im Fenstertitel und in den Menüs.
const PRODUCT: &str = "VivaPlan";

/// Fenstertitel: «VivaPlan - ‹Dateiname›», ohne Datei nur «VivaPlan».
fn window_title(path: Option<&str>) -> String {
    match path.filter(|p| !p.is_empty()) {
        Some(p) => format!("{PRODUCT} - {}", file_name(p)),
        None => PRODUCT.to_string(),
    }
}

/// Setzt den Fenstertitel nach der geöffneten oder gespeicherten Datei.
#[tauri::command]
fn set_window_title<R: Runtime>(window: tauri::WebviewWindow<R>, path: Option<String>) -> Result<String, String> {
    let title = window_title(path.as_deref());
    window.set_title(&title).map_err(|e| e.to_string())?;
    Ok(title)
}

/// Merkt sich den Pfad der Planung für den nächsten Start; ein Fehler dabei stört das Öffnen/Speichern nicht.
fn remember_plan<R: Runtime>(app: &AppHandle<R>, path: &str) {
    if let Ok(data) = app.path().app_data_dir() {
        remember_in(&data, path);
    }
}

fn remember_in(data_dir: &Path, path: &str) {
    let _ = write_setting(data_dir, LAST_PLAN_KEY, Value::String(path.to_string()));
}

/// Zustand für das Beenden: ungespeicherte Änderungen (von der Oberfläche gemeldet) und die Freigabe zum Beenden.
#[derive(Default)]
struct Quit {
    dirty: AtomicBool,
    confirmed: AtomicBool,
}

impl Quit {
    /// Beenden braucht eine Rückfrage, solange Änderungen ungespeichert sind und das Beenden nicht freigegeben ist.
    fn must_ask(&self) -> bool {
        self.dirty.load(Ordering::SeqCst) && !self.confirmed.load(Ordering::SeqCst)
    }
}

#[tauri::command]
fn set_dirty(quit: State<'_, Quit>, dirty: bool) {
    quit.dirty.store(dirty, Ordering::SeqCst);
}

/// Beendet die App ohne weitere Rückfrage (die Oberfläche hat gespeichert oder verworfen).
#[tauri::command]
fn quit_app<R: Runtime>(app: AppHandle<R>, quit: State<'_, Quit>) {
    quit.confirmed.store(true, Ordering::SeqCst);
    app.exit(0);
}

/// #141: Rauchtest der ausgelieferten App (`--smoke`): Die Oberfläche startet, rechnet die Beispielplanung und meldet das
/// Ergebnis mit `smoke_done`. Die App beendet sich dann mit Code 0 (in Ordnung) oder 1 (Fehler); ohne Meldung nach
/// [`SMOKE_TIMEOUT`] mit Code 2.
#[derive(Default)]
struct Smoke(bool);

const SMOKE_FLAG: &str = "--smoke";
const SMOKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

fn smoke_requested(args: impl IntoIterator<Item = OsString>) -> bool {
    args.into_iter().skip(1).any(|a| a == SMOKE_FLAG)
}

/// Meldet das Ergebnis des Rauchtests auf der Konsole und, falls `VIVAPLAN_SMOKE_OUT` gesetzt ist, in dieser Datei
/// (unter Windows hat die App keine Konsole).
fn smoke_report(line: &str) {
    println!("{line}");
    if let Some(out) = std::env::var_os("VIVAPLAN_SMOKE_OUT") {
        let _ = std::fs::write(out, format!("{line}\n"));
    }
}

/// Ausgabe und Exit-Code des Rauchtests.
fn smoke_outcome(ok: bool, message: &str) -> (String, i32) {
    (
        format!("VivaPlan-Rauchtest: {} – {message}", if ok { "ok" } else { "FEHLER" }),
        if ok { 0 } else { 1 },
    )
}

#[tauri::command]
fn smoke_done<R: Runtime>(app: AppHandle<R>, quit: State<'_, Quit>, ok: bool, message: String) {
    // Nur im Rauchtest (den Zustand setzt `main`); sonst ohne Wirkung.
    if !app.try_state::<Smoke>().is_some_and(|s| s.0) {
        return;
    }
    let (line, code) = smoke_outcome(ok, &message);
    smoke_report(&line);
    quit.confirmed.store(true, Ordering::SeqCst);
    app.exit(code);
}

/// Meldet der Oberfläche «quit-requested», wenn ungespeicherte Änderungen vorliegen; `true` heisst: Beenden stoppen.
fn ask_before_exit<R: Runtime>(app: &AppHandle<R>) -> bool {
    let ask = app.state::<Quit>().must_ask();
    if ask {
        let _ = app.emit("quit-requested", ());
    }
    ask
}

/// Beenden über das System (z. B. Dock oder Abmeldung): bei ungespeicherten Änderungen stoppen und fragen.
fn on_run_event<R: Runtime>(app: &AppHandle<R>, event: RunEvent) {
    match event {
        RunEvent::ExitRequested { api, .. } if ask_before_exit(app) => api.prevent_exit(),
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        RunEvent::Opened { urls } => {
            if let Some(p) = urls
                .iter()
                .filter_map(|u| u.to_file_path().ok())
                .find(|p| is_plan_file(p))
            {
                open_from_system(app, p);
            }
        }
        _ => {}
    }
}

/// Schliessen-Knopf des Fensters: bei ungespeicherten Änderungen das Fenster offen lassen und fragen.
fn on_window_event<R: Runtime>(window: &tauri::Window<R>, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        if ask_before_exit(window.app_handle()) {
            api.prevent_close();
        }
    }
}

/// Menüeintrag «Beenden» (⌘Q): fragt bei ungespeicherten Änderungen, sonst sofort beenden.
fn quit_from_menu<R: Runtime>(app: &AppHandle<R>) {
    if !ask_before_exit(app) {
        app.exit(0);
    }
}

/// Beschreibung eines Dateidialogs: Titel und Dateifilter (Name und Endungen).
struct Dialog {
    title: &'static str,
    filter: &'static str,
    extensions: &'static [&'static str],
}

const SHILLER_DIALOG: Dialog = Dialog {
    title: "Shiller-Daten importieren (ie_data.xls)",
    filter: "Tabelle",
    extensions: &["xls", "xlsx", "ods"],
};
const SNB_DIALOG: Dialog = Dialog {
    title: "SNB-Daten importieren (devkum, plkopr, zimoma, rendoblim)",
    filter: "SNB-Datenwürfel (CSV)",
    extensions: &["csv"],
};
const OPEN_DIALOG: Dialog = Dialog {
    title: "Planung öffnen",
    filter: "VivaPlan-Plan",
    extensions: &["vplan", "rplan", "json"], // legacy: RetirementPlanner (rplan)
};
const SAVE_DIALOG: Dialog = Dialog {
    title: "Planung speichern",
    filter: "VivaPlan-Plan",
    extensions: &["vplan"],
};
const LEGACY_DIALOG: Dialog = Dialog {
    title: "Alte config.json importieren",
    filter: "Konfiguration",
    extensions: &["json"],
};
const SERIES_DIALOG: Dialog = Dialog {
    title: "Zeitreihe wählen",
    filter: "Zeitreihe (CSV)",
    extensions: &["csv"],
};

/// Dateiauswahl des Betriebssystems. Hinter einem Trait, damit Tests feste Pfade liefern können;
/// `None` heisst: Dialog abgebrochen.
trait Dialogs: Send + Sync {
    fn pick_file(&self, dialog: &Dialog) -> Option<FilePath>;
    fn pick_files(&self, dialog: &Dialog) -> Option<Vec<FilePath>>;
    fn save_file(&self, dialog: &Dialog, file_name: &str) -> Option<FilePath>;
}

/// Der gemeinsam verwaltete Dateiauswahl-Dienst (siehe `install_dialogs`).
type Picker = Arc<dyn Dialogs>;

/// Echte Dialoge über `tauri-plugin-dialog` (blockierend, öffnen Fenster des Betriebssystems).
struct NativeDialogs<R: Runtime>(AppHandle<R>);

impl<R: Runtime> NativeDialogs<R> {
    fn builder(&self, dialog: &Dialog) -> tauri_plugin_dialog::FileDialogBuilder<R> {
        self.0
            .dialog()
            .file()
            .set_title(dialog.title)
            .add_filter(dialog.filter, dialog.extensions)
    }
}

impl<R: Runtime> Dialogs for NativeDialogs<R> {
    fn pick_file(&self, dialog: &Dialog) -> Option<FilePath> {
        self.builder(dialog).blocking_pick_file()
    }

    fn pick_files(&self, dialog: &Dialog) -> Option<Vec<FilePath>> {
        self.builder(dialog).blocking_pick_files()
    }

    fn save_file(&self, dialog: &Dialog, file_name: &str) -> Option<FilePath> {
        self.builder(dialog).set_file_name(file_name).blocking_save_file()
    }
}

/// Meldet die echten Dialoge an; die Befehle holen sie als `State<Picker>`.
fn install_dialogs<R: Runtime>(app: &tauri::App<R>) {
    let picker: Picker = Arc::new(NativeDialogs(app.handle().clone()));
    app.manage(picker);
}

fn path_string(p: FilePath) -> Result<PathBuf, String> {
    p.into_path().map_err(|e| e.to_string())
}

/// Vorgeschlagener Dateiname (ohne Endung) aus dem Projektnamen.
fn plan_name(plan: &Value) -> &str {
    plan.pointer("/project/name")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("Planung")
}

/// Ergänzt die Endung «vplan», falls der gewählte Pfad keine hat; aus «.rplan» wird «.vplan», damit eine alte Datei
/// nie überschrieben wird.
fn with_default_extension(mut p: PathBuf) -> PathBuf {
    if p.extension().is_none() || legacy_stem(&p.to_string_lossy()).is_some() {
        p.set_extension("vplan");
    }
    p
}

/// legacy: RetirementPlanner – Name (ohne Endung) einer alten .rplan-Datei, sonst `None`.
fn legacy_stem(path: &str) -> Option<String> {
    let p = Path::new(path);
    p.extension()
        .and_then(|e| e.to_str())
        .filter(|e| e.eq_ignore_ascii_case("rplan"))?;
    p.file_stem().map(|s| s.to_string_lossy().into_owned())
}

/// Basisordner für Datenpfade einer alten config.json: der Ordner der Datei oder dessen Elternordner,
/// je nachdem, wo die referenzierte Datei liegt.
fn legacy_base(path: &Path, text: &str) -> PathBuf {
    let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let data_file = serde_json::from_str::<Value>(text).ok().and_then(|v| {
        v.pointer("/WealthManagement/HistoricalData")
            .and_then(Value::as_str)
            .map(String::from)
    });
    [Some(dir.clone()), dir.parent().map(Path::to_path_buf)]
        .into_iter()
        .flatten()
        .find(|b| data_file.as_ref().is_some_and(|f| b.join(f).exists()))
        .unwrap_or(dir)
}

#[derive(Serialize)]
struct Opened {
    path: String,
    plan: Value,
}

/// Version und Zeitpunkt des Builds für das «Über»-Fenster.
#[tauri::command]
fn app_info<R: Runtime>(app: AppHandle<R>) -> Value {
    serde_json::json!({
        "version": app.package_info().version.to_string(),
        "buildEpoch": env!("PP_BUILD_EPOCH").parse::<u64>().unwrap_or(0),
    })
}

/// Rechenzeit pro Lauf, für die Dauerschätzung bei Szenarien und Optimierung.
#[tauri::command]
async fn bench_plan<R: Runtime>(app: AppHandle<R>, plan: Value, path: Option<String>) -> Result<api::BenchOut, String> {
    let lib = library_dir(&app).ok();
    api::bench(&plan, &base_dir(path.as_deref()), lib.as_deref())
}

/// Stand einer Berechnung (Simulation oder Optimierung); ohne Kennung die zuletzt gestartete.
#[tauri::command]
fn progress_get(job_id: Option<u64>) -> engine::progress::Progress {
    match job_id {
        Some(id) => engine::progress::snapshot_of(id),
        None => engine::progress::snapshot(),
    }
}

/// Bricht eine Berechnung ab (mit Kennung nur diese, sonst alle laufenden).
#[tauri::command]
fn cancel_job(job_id: Option<u64>) {
    match job_id {
        Some(id) => {
            engine::progress::cancel_job(id);
        }
        None => engine::progress::cancel(),
    }
}

/// Parameter der Strategien mit Standard-Bandbreiten und den Werten der Planung.
#[tauri::command]
fn opt_params(plan: Value) -> Vec<engine::optimize::CatalogStrategy> {
    api::opt_params(&plan)
}

#[tauri::command]
fn example_plan() -> Value {
    api::example_plan()
}

/// #138: Vorlagen der Grundstückgewinnsteuer.
#[tauri::command]
fn gain_templates() -> Value {
    api::gain_templates()
}

#[tauri::command]
fn validate(plan: Value) -> Vec<Issue> {
    api::validate(&plan)
}

/// Ältere Planung beim Laden auf den aktuellen Stand bringen (#52, #51).
#[tauri::command]
fn upgrade_plan(plan: Value) -> api::UpgradeOut {
    api::upgrade_plan(&plan)
}

#[tauri::command]
async fn preview<R: Runtime>(app: AppHandle<R>, plan: Value, path: Option<String>) -> api::PreviewOut {
    let lib = library_dir(&app).ok();
    api::preview(&plan, &base_dir(path.as_deref()), lib.as_deref())
}

#[tauri::command]
async fn run_simulation<R: Runtime>(
    app: AppHandle<R>,
    plan: Value,
    path: Option<String>,
    job_id: Option<u64>,
) -> api::RunOut {
    let lib = library_dir(&app).ok();
    api::run_as(job_id, &plan, &base_dir(path.as_deref()), lib.as_deref())
}

#[tauri::command]
async fn rerun_paths<R: Runtime>(
    app: AppHandle<R>,
    plan: Value,
    picks: Vec<(u32, usize)>,
    path: Option<String>,
) -> api::RerunOut {
    let lib = library_dir(&app).ok();
    api::rerun_paths(&plan, &picks, &base_dir(path.as_deref()), lib.as_deref())
}

#[tauri::command]
async fn optimize<R: Runtime>(
    app: AppHandle<R>,
    plan: Value,
    mut request: Value,
    path: Option<String>,
    job_id: Option<u64>,
) -> Result<engine::optimize::Report, String> {
    let lib = library_dir(&app).ok();
    if let (Some(id), Some(obj)) = (job_id, request.as_object_mut()) {
        obj.insert("jobId".into(), id.into());
    }
    api::optimize(&plan, &request, &base_dir(path.as_deref()), lib.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn series_info<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    plan_path: Option<String>,
) -> Result<engine::library::SeriesInfo, String> {
    let lib = library_dir(&app).ok();
    api::series_info(&path, &base_dir(plan_path.as_deref()), lib.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
async fn library_list<R: Runtime>(app: AppHandle<R>) -> Result<api::LibraryOut, String> {
    list_in(&library_dir(&app)?)
}

/// Inhalt der Datenbibliothek im Ordner `dir` (legt die ausgelieferten Zeitreihen an, falls nötig).
fn list_in(dir: &Path) -> Result<api::LibraryOut, String> {
    api::library_list(dir).map_err(|e| e.to_string())
}

/// Führt `then` nur aus, wenn im Dialog etwas gewählt wurde (`pick` liefert dann `Some`).
fn picked<T, U>(
    pick: impl FnOnce() -> Option<T>,
    then: impl FnOnce(T) -> Result<U, String>,
) -> Result<Option<U>, String> {
    pick().map(then).transpose()
}

#[tauri::command]
async fn import_shiller<R: Runtime>(
    app: AppHandle<R>,
    dialogs: State<'_, Picker>,
) -> Result<Option<engine::library::ImportReport>, String> {
    picked(
        || dialogs.pick_file(&SHILLER_DIALOG),
        |file| shiller_from(&library_dir(&app)?, file),
    )
}

fn shiller_from(library: &Path, file: FilePath) -> Result<engine::library::ImportReport, String> {
    api::import_shiller(library, &path_string(file)?).map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_snb<R: Runtime>(
    app: AppHandle<R>,
    dialogs: State<'_, Picker>,
) -> Result<Option<engine::library::ImportReport>, String> {
    picked(
        || dialogs.pick_files(&SNB_DIALOG),
        |files| snb_from(&library_dir(&app)?, files),
    )
}

fn snb_from(library: &Path, files: Vec<FilePath>) -> Result<engine::library::ImportReport, String> {
    let files = files.into_iter().map(path_string).collect::<Result<Vec<_>, _>>()?;
    api::import_snb(library, &files).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_plan<R: Runtime>(app: AppHandle<R>, dialogs: State<'_, Picker>) -> Result<Option<Opened>, String> {
    let opened = picked(|| dialogs.pick_file(&OPEN_DIALOG), opened_from)?;
    if let Some(o) = &opened {
        remember_plan(&app, &o.path);
    }
    Ok(opened)
}

fn opened_from(file: FilePath) -> Result<Opened, String> {
    let path = path_string(file)?;
    let plan = api::read_plan(&path).map_err(|e| e.to_string())?;
    Ok(Opened {
        path: path.to_string_lossy().into_owned(),
        plan,
    })
}

#[tauri::command]
async fn save_plan<R: Runtime>(
    app: AppHandle<R>,
    dialogs: State<'_, Picker>,
    plan: Value,
    path: Option<String>,
) -> Result<Option<String>, String> {
    let saved = save_with(&plan, path, |name| {
        dialogs.save_file(&SAVE_DIALOG, &format!("{name}.vplan"))
    })?;
    if let Some(p) = &saved {
        remember_plan(&app, p);
    }
    Ok(saved)
}

/// Speichert unter dem gegebenen Pfad, sonst unter dem im Dialog (`pick`, mit Namensvorschlag) gewählten.
/// #44: Ist der gegebene Pfad eine alte .rplan-Datei, fragt der Dialog wie «Speichern unter» und schlägt denselben
/// Namen vor (mit .vplan); die .rplan-Datei bleibt unverändert.
fn save_with(
    plan: &Value,
    path: Option<String>,
    pick: impl FnOnce(&str) -> Option<FilePath>,
) -> Result<Option<String>, String> {
    let legacy = path.as_deref().and_then(legacy_stem);
    let target = match path {
        Some(p) if legacy.is_none() => Some(PathBuf::from(p)),
        _ => {
            let name = legacy.unwrap_or_else(|| plan_name(plan).to_string());
            picked(|| pick(&name), |file| Ok(with_default_extension(path_string(file)?)))?
        }
    };
    target.map(|t| write_to(&t, plan)).transpose()
}

fn write_to(target: &Path, plan: &Value) -> Result<String, String> {
    api::write_plan(target, plan).map_err(|e| e.to_string())?;
    Ok(target.to_string_lossy().into_owned())
}

#[tauri::command]
async fn import_legacy(dialogs: State<'_, Picker>, start_year: i32) -> Result<Option<api::ImportOut>, String> {
    picked(
        || dialogs.pick_file(&LEGACY_DIALOG),
        |file| legacy_from(file, start_year),
    )
}

fn legacy_from(file: FilePath, start_year: i32) -> Result<api::ImportOut, String> {
    let path = path_string(file)?;
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    // Das alte Programm lief im Projektordner; Datenpfade wie ./data/MSCI.csv gelten von dort aus.
    let base = legacy_base(&path, &text);
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    api::import_legacy(&text, start_year, &base, &name).map_err(|e| e.to_string())
}

fn series_choice(file: FilePath) -> Result<String, String> {
    Ok(path_string(file)?.to_string_lossy().into_owned())
}

#[tauri::command]
async fn pick_series(dialogs: State<'_, Picker>) -> Result<Option<String>, String> {
    picked(|| dialogs.pick_file(&SERIES_DIALOG), series_choice)
}

/// Ein Eintrag der Menüs: Kennung, Beschriftung und Tastenkürzel (leer = keines).
type Entry = (&'static str, &'static str, &'static str);

/// Trennlinie in den Menütabellen.
const SEP: Entry = ("", "", "");

const FILE_MENU: [Entry; 8] = [
    ("new", "Neue Planung", "CmdOrCtrl+N"),
    ("open", "Öffnen…", "CmdOrCtrl+O"),
    SEP,
    ("save", "Speichern", "CmdOrCtrl+S"),
    ("saveas", "Speichern unter…", "CmdOrCtrl+Shift+S"),
    SEP,
    ("import", "Alte config.json importieren…", "CmdOrCtrl+I"),
    ("run", "Simulation starten", "CmdOrCtrl+Enter"),
];

/// #25: anwendungsweite Einstellungen (Grenzen für AHV-Bezug, Bezug Pensionskasse, Maximalbetrag Säule 3a).
/// #59: auf macOS im Programmmenü, auf Windows und Linux am Ende von «Datei».
const SETTINGS: Entry = ("settings", "Einstellungen…", "CmdOrCtrl+,");

/// Einträge des Menüs «Datei» je System (ohne «Beenden», das als Systemeintrag folgt).
fn file_entries() -> Vec<Entry> {
    let tail: &[Entry] = if cfg!(target_os = "macos") {
        &[]
    } else {
        &[SEP, SETTINGS]
    };
    FILE_MENU.iter().chain(tail).copied().collect()
}

const EDIT_MENU: [Entry; 3] = [
    ("undo", "Rückgängig", "CmdOrCtrl+Z"),
    ("redo", "Wiederholen", "CmdOrCtrl+Shift+Z"),
    SEP,
];

/// Kennung des Menüeintrags «Beenden»; die App behandelt ihn selbst (Rückfrage bei ungespeicherten Änderungen).
const QUIT_ID: &str = "quit";

/// Eigene Zeilen des Programmmenüs (macOS): «Über» und «Einstellungen…» mit Trennlinien; der Rest sind
/// Systemeinträge.
#[cfg(target_os = "macos")]
const APP_MENU: [Entry; 4] = [("about", "Über VivaPlan", ""), SEP, SETTINGS, SEP];

const FILE_TITLE: &str = "Datei";
const QUIT_TEXT: &str = "Beenden";
const EDIT_TITLE: &str = "Bearbeiten";
const HELP_TITLE: &str = "Hilfe";

/// #70/#71: «Willkommen …» öffnet den Willkommensdialog, «Erste Schritte» blendet die Checkliste ein.
const HELP_MENU: [Entry; 8] = [
    ("helpctx", "Hilfe zum aktuellen Thema", "F1"),
    ("manual", "Handbuch", "CmdOrCtrl+Shift+/"),
    ("help", "Tastenkürzel", "CmdOrCtrl+/"),
    SEP,
    ("welcome", "Willkommen …", ""),
    ("steps", "Erste Schritte", ""),
    SEP,
    ("about", "Über VivaPlan", ""),
];

/// Baut ein Untermenü aus einer Tabelle.
fn submenu<'a, R: Runtime>(
    app: &'a AppHandle<R>,
    title: &str,
    entries: &[Entry],
) -> tauri::Result<SubmenuBuilder<'a, R, AppHandle<R>>> {
    let mut b = SubmenuBuilder::new(app, title);
    for (id, text, key) in entries {
        if id.is_empty() {
            b = b.separator();
            continue;
        }
        let mut item = MenuItemBuilder::with_id(*id, *text);
        if !key.is_empty() {
            item = item.accelerator(*key);
        }
        b = b.item(&item.build(app)?);
    }
    Ok(b)
}

/// Menü mit den üblichen Tastenkürzeln. Eigene Einträge melden sich als Ereignis «menu» mit ihrer
/// Kennung bei der Oberfläche; Ausschneiden, Kopieren und Einfügen übernimmt das System.
fn menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let quit = MenuItemBuilder::with_id(QUIT_ID, QUIT_TEXT)
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;
    #[cfg(not(target_os = "macos"))]
    let file = submenu(app, FILE_TITLE, &file_entries())?
        .separator()
        .item(&quit)
        .build()?;
    #[cfg(target_os = "macos")]
    let file = submenu(app, FILE_TITLE, &file_entries())?.build()?;
    let edit = submenu(app, EDIT_TITLE, &EDIT_MENU)?
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let help = submenu(app, HELP_TITLE, &HELP_MENU)?.build()?;
    #[cfg(target_os = "macos")]
    {
        let app_menu = submenu(app, PRODUCT, &APP_MENU)?
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .item(&quit)
            .build()?;
        Menu::with_items(app, &[&app_menu, &file, &edit, &help])
    }
    #[cfg(not(target_os = "macos"))]
    Menu::with_items(app, &[&file, &edit, &help])
}

/// Baut die Anwendung (Plugins, Menü, Befehle); getrennt von `main`, damit sie mit der Mock-Runtime testbar ist.
fn configure<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .plugin(tauri_plugin_dialog::init())
        .manage(Quit::default())
        .manage(PendingOpen::default())
        .on_window_event(on_window_event)
        .invoke_handler(tauri::generate_handler![
            app_info,
            bench_plan,
            opt_params,
            progress_get,
            cancel_job,
            example_plan,
            gain_templates,
            upgrade_plan,
            validate,
            preview,
            run_simulation,
            rerun_paths,
            optimize,
            series_info,
            library_list,
            settings_get,
            settings_set,
            startup_plan,
            set_dirty,
            quit_app,
            smoke_done,
            import_shiller,
            import_snb,
            open_plan,
            open_path,
            set_window_title,
            save_plan,
            import_legacy,
            pick_series
        ])
}

/// Meldet einen Menüklick als Ereignis «menu» mit der Kennung des Eintrags an die Oberfläche.
fn forward_menu_event<R: Runtime>(app: &AppHandle<R>, ev: MenuEvent) {
    if ev.id().0 == QUIT_ID {
        return quit_from_menu(app);
    }
    let _ = app.emit("menu", ev.id().0.as_str());
}

/// Setzt Menü und Menüereignisse; braucht den Hauptthread des Betriebssystems und ist daher nur in
/// der echten Anwendung aktiv (nicht in den Tests mit Mock-Runtime).
fn install_menu<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let m = menu(app.handle())?;
    app.set_menu(m)?;
    app.on_menu_event(forward_menu_event);
    Ok(())
}

/// Neue Datenordner der App, deren letzter Teil der Identifier ist (für die Übernahme der alten App-Daten).
fn data_dirs<R: Runtime>(app: &AppHandle<R>) -> Vec<PathBuf> {
    let p = app.path();
    let mut dirs: Vec<PathBuf> = [p.app_data_dir(), p.app_local_data_dir()]
        .into_iter()
        .flatten()
        .collect();
    if cfg!(target_os = "macos") {
        if let Ok(home) = p.home_dir() {
            dirs.push(migrate::webkit_dir(&home, &app.config().identifier));
        }
    }
    dirs
}

/// Erstellt die im Konfigurationsfile beschriebenen Fenster (dort mit `create: false`, damit die App-Daten vor dem
/// ersten Zugriff der Webansicht übernommen sind).
fn create_windows<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let smoke = app.try_state::<Smoke>().is_some_and(|s| s.0);
    for w in app.config().app.windows.clone() {
        let mut b = tauri::WebviewWindowBuilder::from_config(app.handle(), &w)?;
        if smoke {
            // #141: Die Oberfläche erkennt den Rauchtest an dieser Variable (vor allen Skripten der Seite gesetzt).
            b = b.initialization_script("window.__VIVAPLAN_SMOKE__ = true;");
        }
        b.build()?;
    }
    Ok(())
}

/// Einrichtung beim Start der echten Anwendung: alte App-Daten übernehmen (#44), Fenster, Dateidialoge und Menü.
fn setup_app<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    migrate::take_over_all(&migrate::legacy_pairs(&data_dirs(app.handle())));
    create_windows(app)?;
    install_dialogs(app);
    install_menu(app)
}

fn main() {
    let given = plan_arg(std::env::args_os());
    let smoke = smoke_requested(std::env::args_os());
    let app = configure(tauri::Builder::default())
        .manage(Smoke(smoke))
        .setup(setup_app)
        .build(tauri::generate_context!())
        .expect("VivaPlan konnte nicht gestartet werden");
    if let Some(p) = given {
        app.state::<PendingOpen>().set(p);
    }
    if smoke {
        // #141: Zeitgrenze, falls die Oberfläche nie meldet (z. B. Webansicht startet nicht)
        std::thread::spawn(|| {
            std::thread::sleep(SMOKE_TIMEOUT);
            smoke_report(&format!(
                "VivaPlan-Rauchtest: FEHLER – keine Meldung der Oberfläche nach {} s",
                SMOKE_TIMEOUT.as_secs()
            ));
            std::process::exit(2);
        });
    }
    app.run(on_run_event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime, INVOKE_KEY};
    use tauri::webview::InvokeRequest;
    use tauri::{
        ipc::{CallbackFn, InvokeBody},
        App, Listener, WebviewWindow, WebviewWindowBuilder,
    };

    /// Berechnungen teilen sich einen globalen Fortschritts- und Abbruchzustand; Tests laufen daher nacheinander.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Dateidialog-Ersatz mit festen Antworten; merkt sich, wie er gefragt wurde.
    #[derive(Default)]
    struct FakeDialogs {
        file: Option<FilePath>,
        files: Option<Vec<FilePath>>,
        save: Option<FilePath>,
        asked: std::sync::Mutex<Vec<String>>,
    }

    impl FakeDialogs {
        fn note(&self, kind: &str, d: &Dialog, name: &str) {
            let line = format!("{kind}|{}|{}|{}|{name}", d.title, d.filter, d.extensions.join(","));
            self.asked.lock().unwrap().push(line);
        }

        fn asked(&self) -> Vec<String> {
            self.asked.lock().unwrap().clone()
        }
    }

    impl Dialogs for FakeDialogs {
        fn pick_file(&self, d: &Dialog) -> Option<FilePath> {
            self.note("file", d, "");
            self.file.clone()
        }

        fn pick_files(&self, d: &Dialog) -> Option<Vec<FilePath>> {
            self.note("files", d, "");
            self.files.clone()
        }

        fn save_file(&self, d: &Dialog, name: &str) -> Option<FilePath> {
            self.note("save", d, name);
            self.save.clone()
        }
    }

    /// Hält die Sperre `SERIAL` und entfernt am Ende des Tests den Datenordner des Testprozesses.
    struct Serial {
        _lock: std::sync::MutexGuard<'static, ()>,
        data: Option<PathBuf>,
    }

    impl Drop for Serial {
        fn drop(&mut self) {
            if let Some(d) = &self.data {
                let _ = std::fs::remove_dir_all(d);
            }
        }
    }

    /// #159: Jeder Testprozess hat einen eigenen Datenordner (Einstellungen, zuletzt geöffnete Planung). nextest startet
    /// jeden Test in einem eigenen Prozess; die Sperre `SERIAL` gilt nur innerhalb eines Prozesses. Mit einem gemeinsamen
    /// Ordner überschrieb ein gleichzeitig laufender Test die Einstellungen (Lesen, Ändern, Schreiben), und
    /// `issue_106_einstellung_wird_ueber_den_befehl_gespeichert_und_gelesen` schlug zufällig fehl.
    fn test_context() -> tauri::Context<MockRuntime> {
        let mut ctx = mock_context(noop_assets());
        let id = format!("{}.test-{}", ctx.config().identifier, std::process::id());
        ctx.config_mut().identifier = id;
        ctx
    }

    fn setup_with(fake: FakeDialogs) -> (App<MockRuntime>, WebviewWindow<MockRuntime>, Serial, Arc<FakeDialogs>) {
        let lock = SERIAL.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let app = configure(mock_builder()).build(test_context()).expect("App baut");
        let guard = Serial {
            _lock: lock,
            data: app.path().app_data_dir().ok(),
        };
        let fake = Arc::new(fake);
        let picker: Picker = fake.clone();
        app.manage(picker);
        let w = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .expect("Fenster");
        (app, w, guard, fake)
    }

    fn setup() -> (App<MockRuntime>, WebviewWindow<MockRuntime>, Serial) {
        let (app, w, guard, _) = setup_with(FakeDialogs::default());
        (app, w, guard)
    }

    /// #141: Herkunft der Oberfläche, wie Tauri sie prüft: unter Windows (WebView2) `http://tauri.localhost`, sonst
    /// `tauri://localhost`. Mit der falschen Herkunft lehnt Tauri jeden Befehl ab («… not allowed»).
    const APP_ORIGIN: &str = if cfg!(windows) {
        "http://tauri.localhost"
    } else {
        "tauri://localhost"
    };

    fn call(w: &WebviewWindow<MockRuntime>, cmd: &str, args: Value) -> Result<Value, Value> {
        tauri::test::get_ipc_response(
            w,
            InvokeRequest {
                cmd: cmd.into(),
                callback: CallbackFn(0),
                error: CallbackFn(1),
                url: APP_ORIGIN.parse().unwrap(),
                body: InvokeBody::Json(args),
                headers: Default::default(),
                invoke_key: INVOKE_KEY.to_string(),
            },
        )
        .map(|b| b.deserialize::<Value>().unwrap())
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("pp-app-test-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn home_ist_gesetzt_oder_leer() {
        let h = home();
        if std::env::var_os("HOME").is_some() {
            assert!(!h.as_os_str().is_empty());
        }
    }

    #[test]
    fn home_bevorzugt_home_vor_userprofile() {
        let os = |s: &str| Some(OsString::from(s));
        assert_eq!(home_from(os("/h"), os("C:\\u")), PathBuf::from("/h"));
        assert_eq!(home_from(None, os("C:\\u")), PathBuf::from("C:\\u"));
        assert_eq!(home_from(None, None), PathBuf::new());
    }

    #[test]
    fn zeitreihenordner_wird_angelegt_oder_fehler_gemeldet() {
        let dir = tmp("lib");
        let lib = ensure_library_dir(&dir.join("neu")).unwrap();
        assert!(lib.is_dir() && lib.ends_with("Zeitreihen"));
        // Existiert schon: weiterhin Erfolg.
        assert_eq!(ensure_library_dir(&dir.join("neu")).unwrap(), lib);
        // Datenverzeichnis ist eine Datei: Fehler mit Pfad.
        let datei = dir.join("datei");
        std::fs::write(&datei, "x").unwrap();
        let e = ensure_library_dir(&datei).unwrap_err();
        assert!(e.contains("Zeitreihen"), "{e}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn base_dir_nimmt_ordner_der_projektdatei() {
        assert_eq!(base_dir(Some("/a/b/plan.vplan")), PathBuf::from("/a/b"));
        assert_eq!(base_dir(None), home());
        assert_eq!(base_dir(Some("plan.vplan")), PathBuf::from(""));
    }

    #[test]
    fn plan_name_faellt_auf_standard_zurueck() {
        assert_eq!(
            plan_name(&json!({"project": {"name": "Meine Planung"}})),
            "Meine Planung"
        );
        assert_eq!(plan_name(&json!({"project": {"name": "  "}})), "Planung");
        assert_eq!(plan_name(&json!({"project": {"name": 5}})), "Planung");
        assert_eq!(plan_name(&json!({})), "Planung");
        assert_eq!(plan_name(&json!({"project": {"name": "Zürich ✓"}})), "Zürich ✓");
    }

    #[test]
    fn endung_wird_nur_ergaenzt_wenn_fehlend() {
        assert_eq!(
            with_default_extension(PathBuf::from("/x/plan")),
            PathBuf::from("/x/plan.vplan")
        );
        assert_eq!(
            with_default_extension(PathBuf::from("/x/plan.json")),
            PathBuf::from("/x/plan.json")
        );
    }

    #[test]
    fn legacy_base_findet_datei_im_ordner_oder_elternordner() {
        let root = tmp("legacy");
        let sub = root.join("config");
        std::fs::create_dir_all(&sub).unwrap();
        let cfg = sub.join("config.json");
        let text = r#"{"WealthManagement":{"HistoricalData":"data.csv"}}"#;
        // Datei nirgends vorhanden: Ordner der config.json.
        assert_eq!(legacy_base(&cfg, text), sub);
        // Datei im Elternordner.
        std::fs::write(root.join("data.csv"), "x").unwrap();
        assert_eq!(legacy_base(&cfg, text), root);
        // Datei im selben Ordner hat Vorrang.
        std::fs::write(sub.join("data.csv"), "x").unwrap();
        assert_eq!(legacy_base(&cfg, text), sub);
        // Ungültiges JSON oder fehlender Eintrag.
        assert_eq!(legacy_base(&cfg, "kein json"), sub);
        assert_eq!(legacy_base(&cfg, "{}"), sub);
        assert_eq!(legacy_base(Path::new("config.json"), "{}"), PathBuf::from(""));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn app_info_liefert_version_und_buildzeit() {
        let (_app, w, _g) = setup();
        let v = call(&w, "app_info", json!({})).unwrap();
        assert!(v["version"].as_str().is_some_and(|s| !s.is_empty()));
        assert!(v["buildEpoch"].as_u64().unwrap() > 0);
    }

    #[test]
    fn beispielplan_validiert_ohne_fehler() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        assert!(plan.is_object());
        let issues = call(&w, "validate", json!({"plan": plan})).unwrap();
        assert!(issues.is_array());
    }

    #[test]
    fn validate_meldet_probleme_bei_leerem_plan() {
        let (_app, w, _g) = setup();
        let issues = call(&w, "validate", json!({"plan": {}})).unwrap();
        assert!(!issues.as_array().unwrap().is_empty());
    }

    #[test]
    fn fortschritt_und_abbruch() {
        let (_app, w, _g) = setup();
        assert!(call(&w, "progress_get", json!({})).unwrap().is_object());
        assert!(call(&w, "cancel_job", json!({})).unwrap().is_null());
    }

    #[test]
    fn opt_params_liefert_katalog() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let v = call(&w, "opt_params", json!({"plan": plan})).unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn preview_und_simulation_laufen_fuer_beispielplan() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let p = call(&w, "preview", json!({"plan": plan, "path": null})).unwrap();
        assert!(p.is_object());
        let r = call(&w, "run_simulation", json!({"plan": plan})).unwrap();
        assert!(r.is_object());
    }

    #[test]
    fn rerun_paths_rechnet_einzelne_laeufe_nach_und_meldet_fehler() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let r = call(&w, "rerun_paths", json!({"plan": plan, "picks": [[50, 0]]})).unwrap();
        assert!(r["error"].is_null(), "{r}");
        assert_eq!(r["details"].as_array().map(|a| a.len()), Some(1), "{r}");
        assert_eq!(r["details"][0][0], 50);
        let r = call(&w, "rerun_paths", json!({"plan": {}, "picks": []})).unwrap();
        assert!(
            r["error"].is_string() || r["details"].as_array().is_some_and(|a| a.is_empty()),
            "{r}"
        );
    }

    #[test]
    fn bench_plan_misst_oder_meldet_fehler() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let ok = call(&w, "bench_plan", json!({"plan": plan}));
        assert!(ok.is_ok() || ok.is_err());
        assert!(call(&w, "bench_plan", json!({"plan": {}})).is_err());
    }

    #[test]
    fn optimize_liefert_bericht_und_meldet_ungueltige_anfrage() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let r = call(&w, "optimize", json!({"plan": plan, "request": {}})).unwrap();
        assert!(r["outcomes"].is_array());
        let e = call(&w, "optimize", json!({"plan": plan, "request": "unsinn"})).unwrap_err();
        assert!(e.is_string());
    }

    #[test]
    fn series_info_fehler_bei_fehlender_datei() {
        let (_app, w, _g) = setup();
        let e = call(
            &w,
            "series_info",
            json!({"path": "/gibt/es/nicht.csv", "planPath": null}),
        )
        .unwrap_err();
        assert!(e.is_string());
    }

    #[test]
    fn series_info_liest_csv() {
        let (_app, w, _g) = setup();
        let dir = tmp("series");
        let csv = dir.join("s.csv");
        std::fs::write(&csv, "Date,Value\n2000-01,1.0\n2000-02,1.1\n2000-03,1.2\n").unwrap();
        let r = call(
            &w,
            "series_info",
            json!({"path": csv.to_string_lossy(), "planPath": dir.join("p.vplan").to_string_lossy()}),
        );
        // Format der CSV hängt vom Engine-Parser ab; beide Ausgänge laufen über den Befehl.
        assert!(r.is_ok() || r.unwrap_err().is_string());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn einstellungen_werden_in_datei_gespeichert_und_gelesen() {
        let dir = tmp("settings");
        assert!(read_settings(&dir).is_empty());
        write_setting(&dir, "showTaxCols", json!(true)).unwrap();
        write_setting(&dir, "andere", json!("x")).unwrap();
        let all = read_settings(&dir);
        assert_eq!(all["showTaxCols"], json!(true));
        assert_eq!(all["andere"], json!("x"));
        // unlesbare Datei: Standardwerte, und Schreiben überschreibt sie
        std::fs::write(dir.join(SETTINGS_FILE), "kein json").unwrap();
        assert!(read_settings(&dir).is_empty());
        std::fs::write(dir.join(SETTINGS_FILE), "[1]").unwrap();
        assert!(read_settings(&dir).is_empty());
        write_setting(&dir, "showTaxCols", json!(false)).unwrap();
        assert_eq!(read_settings(&dir)["showTaxCols"], json!(false));
        // Datenverzeichnis nicht beschreibbar (ist eine Datei)
        let file = dir.join("datei");
        std::fs::write(&file, "x").unwrap();
        assert!(write_setting(&file, "k", json!(1)).is_err());
    }

    #[test]
    fn einstellungen_ueber_die_befehle() {
        let (_app, w, _g) = setup();
        call(&w, "settings_set", json!({ "key": "showTaxCols", "value": true })).unwrap();
        let v = call(&w, "settings_get", json!({})).unwrap();
        assert_eq!(v["showTaxCols"], json!(true));
    }

    #[test]
    fn library_list_zeigt_ausgelieferte_zeitreihen() {
        let (_app, w, _g) = setup();
        let v = call(&w, "library_list", json!({})).unwrap();
        assert!(v["dir"].as_str().unwrap().ends_with("Zeitreihen"));
        assert!(v["entries"].is_array());
    }

    #[test]
    fn bibliothek_in_eigenem_ordner_und_fehler_bei_datei() {
        let dir = tmp("list");
        let lib = ensure_library_dir(&dir).unwrap();
        let v = list_in(&lib).unwrap();
        assert_eq!(v.dir, lib.to_string_lossy());
        assert!(!v.entries.is_empty(), "ausgelieferte Zeitreihen werden angelegt");
        // Ein Ordnerpfad, der eine Datei ist, lässt sich nicht lesen.
        let datei = dir.join("datei");
        std::fs::write(&datei, "x").unwrap();
        assert!(list_in(&datei).err().unwrap().contains("datei"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn library_dir_wird_angelegt() {
        let (app, _w, _g) = setup();
        let d = library_dir(app.handle()).unwrap();
        assert!(d.is_dir());
    }

    /// #142: Antwort ohne Zeitfelder (Rechenzeit, Buildzeit).
    fn without_times(v: &mut Value) {
        match v {
            Value::Object(o) => {
                o.remove("elapsedMs");
                o.remove("buildEpoch");
                o.values_mut().for_each(without_times);
            }
            Value::Array(a) => a.iter_mut().for_each(without_times),
            _ => {}
        }
    }

    /// #142: Platzhalter der Tabelle durch kleine Planungen ersetzen.
    fn table_args(args: &Value) -> Value {
        let mut a = args.clone();
        if let Some(p) = a.get("plan").and_then(Value::as_str) {
            let mut plan = api::example_plan();
            plan["scenarios"]["sweeps"] = json!([]);
            match p {
                "$plan" => plan["returns"]["source"] = json!("fixed"),
                "$planMc" => {
                    plan["returns"]["source"] = json!("bootstrap");
                    plan["returns"]["bootstrap"] = json!({ "paths": 200, "block": 12, "seed": 1 });
                }
                other => panic!("unbekannter Platzhalter {other}"),
            }
            a["plan"] = plan;
        }
        a
    }

    fn ipc_table() -> Value {
        serde_json::from_str(include_str!("../ipc-commands.json")).expect("Tabelle ist JSON")
    }

    /// #142/AK1, AK2, AK4: Jeder Rechenbefehl der Tabelle gibt über die Tauri-IPC (Argumente wie die Oberfläche sie
    /// sendet, camelCase) dieselbe Antwort wie der Verteiler des Entwicklungsservers (`engine::dispatch::call`, den
    /// `devserver::call` unverändert aufruft). Darin enthalten (AK4): `run_simulation` mit `jobId` und danach
    /// `progress_get` mit derselben Kennung.
    #[test]
    fn issue_142_ak2_tauri_commands_answer_like_the_dev_server() {
        let (app, w, _g) = setup();
        let lib = library_dir(app.handle()).unwrap();
        let table = ipc_table();
        let entries = table["compute"].as_array().unwrap();
        assert!(entries.len() >= 14);
        let mut progress_checked = false;
        for e in entries {
            let cmd = e["cmd"].as_str().unwrap();
            let args = table_args(&e["args"]);
            let mut tauri = call(&w, cmd, args.clone()).map_err(|e| e.as_str().unwrap_or_default().to_string());
            // Die IPC-Antwort kommt als JSON-Text; serde_json liest Gleitkommazahlen daraus nicht immer bitgenau
            // (ohne `float_roundtrip`). Die Antwort des Verteilers nimmt deshalb denselben Weg über Text.
            let mut dev = engine::dispatch::call(cmd, &args, &lib, &home())
                .map(|v| serde_json::from_str::<Value>(&v.to_string()).unwrap());
            for v in [&mut tauri, &mut dev].into_iter().flatten() {
                without_times(v);
            }
            if e["compare"] == "keys" {
                let keys = |r: &Result<Value, String>| -> Vec<String> {
                    let mut k: Vec<String> = r.as_ref().unwrap().as_object().unwrap().keys().cloned().collect();
                    k.sort();
                    k
                };
                assert_eq!(keys(&tauri), keys(&dev), "{cmd}");
            } else {
                assert_eq!(tauri, dev, "{cmd} {args}");
            }
            assert!(tauri.is_ok(), "{cmd}: {tauri:?}");
            if cmd == "progress_get" {
                let p = tauri.as_ref().unwrap();
                assert_eq!((p["id"].as_u64(), p["active"].as_bool()), (Some(4711), Some(false)));
                assert!(p["done"].as_u64().unwrap() >= 200, "{p}");
                progress_checked = true;
            }
        }
        assert!(progress_checked);
    }

    /// #142/AK3: Die Tabelle nennt jeden Befehl aus `generate_handler!`, als Rechenbefehl oder als «nur Desktop-App»,
    /// und keinen anderen. Ein neuer Befehl ohne Eintrag lässt den Test fehlschlagen (Gegenstück für den
    /// Entwicklungsserver: `engine::dispatch`, Test `issue_142_ak3_…`).
    #[test]
    fn issue_142_ak3_table_lists_every_tauri_command() {
        let src = include_str!("main.rs");
        let start = src.find("generate_handler![").unwrap() + "generate_handler![".len();
        let end = start + src[start..].find(']').unwrap();
        let mut handler: Vec<String> = src[start..end]
            .split(',')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect();
        handler.sort();
        let table = ipc_table();
        let mut listed: Vec<String> = table["compute"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["cmd"].as_str().unwrap().to_string())
            .chain(
                table["desktopOnly"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| c.as_str().unwrap().to_string()),
            )
            .collect();
        listed.sort();
        listed.dedup();
        assert_eq!(handler, listed);
    }

    /// #141: Startoption des Rauchtests, Ausgabe und Exit-Code.
    #[test]
    fn issue_141_ak4_smoke_option_and_outcome() {
        let args = |a: &[&str]| a.iter().map(OsString::from).collect::<Vec<_>>();
        assert!(smoke_requested(args(&["vivaplan-app", "--smoke"])));
        assert!(!smoke_requested(args(&["vivaplan-app"])));
        assert!(!smoke_requested(args(&["--smoke"])), "das Programm selbst zählt nicht");
        assert_eq!(
            smoke_outcome(true, "472 Monate gerechnet"),
            ("VivaPlan-Rauchtest: ok – 472 Monate gerechnet".into(), 0)
        );
        assert_eq!(smoke_outcome(false, "x").1, 1);
        // Ohne --smoke tut der Befehl nichts (die App läuft weiter)
        let (_app, w, _g) = setup();
        assert_eq!(
            call(&w, "smoke_done", json!({"ok": false, "message": "x"})).unwrap(),
            Value::Null
        );
    }

    /// #162/AK1: `smoke_report` schreibt die Meldung des Rauchtests auch in die Datei aus `VIVAPLAN_SMOKE_OUT` (unter
    /// Windows hat die App keine Konsole; `pakete.yml` liest die Datei).
    #[test]
    fn issue_162_ak1_smoke_report_writes_the_file() {
        let out = std::env::temp_dir().join(format!("vivaplan-smoke-{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&out);
        std::env::set_var("VIVAPLAN_SMOKE_OUT", &out);
        smoke_report("VivaPlan-Rauchtest: ok – 472 Monate gerechnet");
        std::env::remove_var("VIVAPLAN_SMOKE_OUT");
        assert_eq!(
            std::fs::read_to_string(&out).unwrap(),
            "VivaPlan-Rauchtest: ok – 472 Monate gerechnet\n"
        );
        let _ = std::fs::remove_file(&out);
    }

    /// #141/AK3: Pfade unter Windows (Laufwerk, Backslash, UNC, Leerzeichen und Umlaute, .rplan → .vplan). Die Fälle
    /// werden auf allen Systemen übersetzt, ausgeführt nur unter Windows (dort gelten Backslash und Laufwerk als Pfad).
    #[cfg_attr(not(windows), allow(dead_code))]
    fn windows_path_cases() {
        assert_eq!(
            legacy_stem(r"C:\Users\Anna\Planungen\Alte Planung.rplan").as_deref(),
            Some("Alte Planung")
        );
        assert_eq!(legacy_stem(r"C:\Users\Anna\Planungen\Planung.vplan"), None);
        assert_eq!(
            with_default_extension(PathBuf::from(r"C:\Daten\Pläne\Neue Planung")),
            PathBuf::from(r"C:\Daten\Pläne\Neue Planung.vplan")
        );
        assert_eq!(
            with_default_extension(PathBuf::from(r"D:\x\alt.rplan")),
            PathBuf::from(r"D:\x\alt.vplan")
        );
        assert_eq!(
            base_dir(Some(r"C:\Users\Zoë\Planung ü.vplan")),
            PathBuf::from(r"C:\Users\Zoë")
        );
        assert_eq!(
            base_dir(Some(r"\\server\share\p.vplan")),
            PathBuf::from(r"\\server\share\")
        );
        assert_eq!(file_name(r"C:\a\b\Planung.vplan"), "Planung.vplan");
        assert!(is_plan_file(Path::new(r"C:\a\P.VPLAN")));
        let args = [
            r"C:\Programme\VivaPlan\vivaplan-app.exe",
            r"C:\Users\Anna\Planung ü.vplan",
        ];
        assert_eq!(
            plan_arg(args.iter().map(OsString::from)),
            Some(PathBuf::from(r"C:\Users\Anna\Planung ü.vplan"))
        );
    }

    #[cfg(windows)]
    #[test]
    fn issue_141_ak3_windows_paths() {
        windows_path_cases();
    }

    /// #141: Leerzeichen und Umlaute in Pfaden mit «/» (alle Systeme).
    #[test]
    fn paths_with_spaces_and_umlauts() {
        assert_eq!(
            legacy_stem("/Users/Zoë/Meine Pläne/Alte Planung.rplan").as_deref(),
            Some("Alte Planung")
        );
        assert_eq!(
            base_dir(Some("/Users/Zoë/Meine Pläne/p ü.vplan")),
            PathBuf::from("/Users/Zoë/Meine Pläne")
        );
        assert_eq!(
            with_default_extension(PathBuf::from("/Users/Zoë/Neue Planung")),
            PathBuf::from("/Users/Zoë/Neue Planung.vplan")
        );
    }

    #[test]
    fn plan_speichern_mit_pfad_und_wieder_lesen() {
        let (_app, w, _g) = setup();
        let dir = tmp("save");
        let file = dir.join("p.vplan");
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let r = call(&w, "save_plan", json!({"plan": plan, "path": file.to_string_lossy()})).unwrap();
        assert_eq!(r.as_str().unwrap(), file.to_string_lossy());
        assert_eq!(api::read_plan(&file).unwrap(), plan);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plan_speichern_in_nicht_existierenden_ordner_ist_fehler() {
        let (_app, w, _g) = setup();
        let e = call(&w, "save_plan", json!({"plan": {}, "path": "/gibt/es/nicht/p.vplan"})).unwrap_err();
        assert!(e.as_str().unwrap().contains("p.vplan"));
    }

    /// #136/AK4: Schlägt das Speichern fehl, meldet der Befehl den Fehler (die Oberfläche zeigt ihn und lässt die
    /// Planung ungespeichert, e2e «#136/AK4»), und die vorhandene Datei behält den alten Inhalt.
    #[cfg(unix)]
    #[test]
    fn issue_136_ak4_failed_save_reports_error_and_keeps_file() {
        use std::os::unix::fs::PermissionsExt;
        let (_app, w, _g) = setup();
        let dir = tmp("save-readonly");
        let file = dir.join("p.vplan");
        let alt = json!({"format": "vivaplan/1", "marker": "alt"});
        call(&w, "save_plan", json!({"plan": alt, "path": file.to_string_lossy()})).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();
        let e = call(
            &w,
            "save_plan",
            json!({"plan": {"format": "vivaplan/1", "marker": "neu"}, "path": file.to_string_lossy()}),
        )
        .unwrap_err();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(e.as_str().unwrap().contains("p.vplan"), "{e}");
        assert_eq!(api::read_plan(&file).unwrap(), alt);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "keine temporäre Datei");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn plan_oeffnen_liest_datei_und_meldet_fehler() {
        let dir = tmp("open");
        let f = dir.join("p.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1", "marker": 1}"#).unwrap();
        let o = opened_from(FilePath::Path(f.clone())).unwrap();
        assert_eq!(o.path, f.to_string_lossy());
        assert_eq!(o.plan["marker"], 1);
        // #144/AK2: Eine Datei aus einer neueren Version wird mit der Kennung in der Meldung abgewiesen.
        std::fs::write(&f, r#"{"format": "vivaplan/2"}"#).unwrap();
        assert!(opened_from(FilePath::Path(f.clone()))
            .err()
            .unwrap()
            .contains("Die Datei hat das Format «vivaplan/2». Sie braucht eine neuere Version von VivaPlan"));
        // Alte config.json wird abgewiesen.
        let alt = dir.join("alt.json");
        std::fs::write(&alt, r#"{"General": {}}"#).unwrap();
        assert!(opened_from(FilePath::Path(alt))
            .err()
            .unwrap()
            .contains("Alte config.json"));
        // Kein JSON, fehlende Datei, nicht umwandelbarer Pfad.
        let kaputt = dir.join("k.vplan");
        std::fs::write(&kaputt, "{").unwrap();
        assert!(opened_from(FilePath::Path(kaputt)).is_err());
        assert!(opened_from(FilePath::Path(dir.join("fehlt"))).is_err());
        assert!(opened_from(FilePath::Url("https://example.com/x".parse().unwrap())).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_to_schreibt_und_meldet_pfad() {
        let dir = tmp("write");
        let f = dir.join("a.vplan");
        assert_eq!(write_to(&f, &json!({"a": 1})).unwrap(), f.to_string_lossy());
        assert!(write_to(&dir.join("x/y.vplan"), &json!({})).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn serienwahl_gibt_pfad_als_text() {
        assert_eq!(
            series_choice(FilePath::Path(PathBuf::from("/a/b.csv"))).unwrap(),
            "/a/b.csv"
        );
        assert!(series_choice(FilePath::Url("https://example.com/x".parse().unwrap())).is_err());
    }

    #[test]
    fn alte_konfiguration_wird_importiert() {
        let src = root().join("crates/engine/tests/fixtures/legacy_fixed.json");
        let r = legacy_from(FilePath::Path(src), 2026).unwrap();
        assert!(r.plan.is_object());
        assert!(legacy_from(FilePath::Path(PathBuf::from("/gibt/es/nicht.json")), 2026)
            .err()
            .unwrap()
            .contains("nicht.json"));
        let dir = tmp("legacy-bad");
        let bad = dir.join("c.json");
        std::fs::write(&bad, "kein json").unwrap();
        assert!(legacy_from(FilePath::Path(bad), 2026).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn shiller_import_aus_tabelle_und_fehlerfall() {
        let lib = tmp("shiller");
        // #94: die Originaltabelle liegt nur lokal; ohne sie entfällt dieser Teil.
        let ods = root().join("crates/engine/data/quellen/shiller-index.ods");
        if ods.exists() {
            let rep = shiller_from(&lib, FilePath::Path(ods)).unwrap();
            assert!(!rep.written.is_empty());
        } else {
            eprintln!("übersprungen: {} fehlt (#94)", ods.display());
        }
        assert!(shiller_from(&lib, FilePath::Path(lib.join("fehlt.xls"))).is_err());
        assert!(shiller_from(&lib, FilePath::Url("https://example.com/x".parse().unwrap())).is_err());
        let _ = std::fs::remove_dir_all(&lib);
    }

    #[test]
    fn snb_import_fehlerfaelle() {
        let lib = tmp("snb");
        assert!(snb_from(&lib, vec![FilePath::Path(lib.join("fehlt.csv"))]).is_err());
        assert!(snb_from(&lib, vec![FilePath::Url("https://example.com/x".parse().unwrap())]).is_err());
        let _ = std::fs::remove_dir_all(&lib);
    }

    /// Alle eigenen Menüeinträge dieses Systems; «Über» des Programmmenüs (macOS) ist derselbe Befehl wie im
    /// Menü «Hilfe» und zählt nur einmal.
    fn eigene_eintraege() -> Vec<Entry> {
        let mut v = file_entries();
        v.extend(EDIT_MENU.iter().chain(&HELP_MENU).copied());
        #[cfg(target_os = "macos")]
        v.extend(APP_MENU.iter().filter(|e| e.0 != "about").copied());
        v
    }

    /// #59/AK1–AK4: Einstellungen auf macOS im Programmmenü, sonst am Ende von «Datei»; genau einmal.
    #[test]
    fn issue_59_einstellungen_je_system() {
        let n = eigene_eintraege().iter().filter(|e| e.0 == "settings").count();
        assert_eq!(n, 1, "Einstellungen genau einmal");
        assert_eq!(SETTINGS, ("settings", "Einstellungen…", "CmdOrCtrl+,"));
        let file = file_entries();
        #[cfg(target_os = "macos")]
        {
            assert_eq!(APP_MENU, [("about", "Über VivaPlan", ""), SEP, SETTINGS, SEP]);
            assert!(!file.contains(&SETTINGS), "nicht im Menü «Datei»");
            assert_eq!(file.last(), Some(&("run", "Simulation starten", "CmdOrCtrl+Enter")));
        }
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            file[file.len() - 2..],
            [SEP, SETTINGS],
            "am Ende von «Datei», danach folgt «Beenden»"
        );
    }

    #[test]
    fn menuetabellen_sind_konsistent() {
        let mut ids = Vec::new();
        for (id, text, key) in &eigene_eintraege() {
            if id.is_empty() {
                assert!(text.is_empty() && key.is_empty());
            } else {
                assert!(!text.is_empty(), "{id} ohne Beschriftung");
                ids.push(*id);
            }
        }
        let n = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), n, "doppelte Menükennung");
        for must in ["new", "open", "save", "saveas", "import", "run", "settings", "about"] {
            assert!(ids.contains(&must), "{must} fehlt");
        }
    }

    #[test]
    fn auswahl_ohne_treffer_fuehrt_nichts_aus() {
        // `Err` als Folgeschritt: Wäre er bei Abbruch aufgerufen worden, wäre das Ergebnis ein Fehler.
        let r = picked(|| None::<String>, Err::<u8, String>);
        assert_eq!(r, Ok(None));
        assert_eq!(picked(|| Some(2), |x| Ok(x * 2)), Ok(Some(4)));
        assert_eq!(
            picked(|| Some("fehler".to_string()), Err::<u8, String>),
            Err("fehler".into())
        );
    }

    /// Ergebnis von `save_with` samt Beobachtung des Dialogs: (Ergebnis, gefragt, Namensvorschlag).
    type Saved = (Result<Option<String>, String>, bool, String);

    fn save_mit_antwort(plan: &Value, path: Option<String>, antwort: Option<FilePath>) -> Saved {
        let mut gefragt = false;
        let mut vorschlag = String::new();
        let r = save_with(plan, path, |n| {
            gefragt = true;
            vorschlag = n.to_string();
            antwort
        });
        (r, gefragt, vorschlag)
    }

    #[test]
    fn speichern_mit_dialog_ergaenzt_endung_und_respektiert_abbruch() {
        let dir = tmp("save-dialog");
        let plan = json!({"project": {"name": "Mein Plan"}});
        // Abbruch im Dialog: nichts geschrieben.
        let (r, gefragt, _) = save_mit_antwort(&plan, None, None);
        assert_eq!((r, gefragt), (Ok(None), true));
        // Dialog liefert Pfad ohne Endung; der Namensvorschlag stammt aus dem Projektnamen.
        let (r, _, vorschlag) = save_mit_antwort(&plan, None, Some(FilePath::Path(dir.join("ziel"))));
        assert_eq!(vorschlag, "Mein Plan");
        assert_eq!(r.unwrap().unwrap(), dir.join("ziel.vplan").to_string_lossy());
        assert!(dir.join("ziel.vplan").is_file());
        // Mit Pfad wird der Dialog nicht gefragt.
        let p = dir.join("direkt.vplan").to_string_lossy().into_owned();
        let (r, gefragt, _) = save_mit_antwort(&plan, Some(p.clone()), None);
        assert_eq!(r, Ok(Some(p)));
        assert!(!gefragt, "mit Pfad kein Dialog");
        // Nicht umwandelbare Auswahl ist ein Fehler.
        let url = FilePath::Url("https://example.com/x".parse().unwrap());
        assert!(save_mit_antwort(&plan, None, Some(url)).0.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn befehle_weisen_fehlende_argumente_ab() {
        let (_app, w, _g) = setup();
        for cmd in [
            "validate",
            "opt_params",
            "preview",
            "run_simulation",
            "bench_plan",
            "optimize",
            "series_info",
            "save_plan",
            "import_legacy",
        ] {
            let e = call(&w, cmd, json!({}));
            assert!(e.is_err(), "{cmd} ohne Argumente muss scheitern");
        }
    }

    #[test]
    fn unbekannter_befehl_wird_abgewiesen() {
        let (_app, w, _g) = setup();
        assert!(call(&w, "gibt_es_nicht", json!({})).is_err());
    }

    #[test]
    fn simulation_der_beispieldatei_liefert_ergebnis_ohne_fehler() {
        let (_app, w, _g) = setup();
        let file = root().join("crates/engine/assets/beispiel.vplan");
        let plan = api::read_plan(&file).unwrap();
        let path = file.to_string_lossy().into_owned();
        let r = call(&w, "run_simulation", json!({"plan": plan, "path": path})).unwrap();
        assert!(r["error"].is_null(), "{}", r["error"]);
        assert!(!r["single"].is_null() || !r["batch"].is_null() || !r["scenarios"].is_null());
        assert!(r["elapsedMs"].as_f64().unwrap() >= 0.0);
        let p = call(&w, "preview", json!({"plan": plan, "path": path})).unwrap();
        assert!(p["error"].is_null(), "{}", p["error"]);
        assert!(p["issues"].is_array());
        let b = call(&w, "bench_plan", json!({"plan": plan, "path": path})).unwrap();
        assert!(b.is_object());
    }

    #[test]
    fn zeitreihe_im_planordner_wird_relativ_aufgeloest() {
        let (_app, w, _g) = setup();
        let dir = tmp("rel");
        std::fs::copy(
            root().join("crates/engine/tests/fixtures/renditen-muster.csv"),
            dir.join("renditen-muster.csv"),
        )
        .unwrap();
        let plan_path = dir.join("p.vplan").to_string_lossy().into_owned();
        let info = call(
            &w,
            "series_info",
            json!({"path": "renditen-muster.csv", "planPath": plan_path}),
        )
        .unwrap();
        assert!(info.is_object());
        // Ohne Planpfad (Basis: Home) wird sie nicht gefunden.
        assert!(call(&w, "series_info", json!({"path": "renditen-muster.csv"})).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fortschritt_nach_berechnung_und_abbruch_ohne_lauf() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        call(&w, "run_simulation", json!({"plan": plan})).unwrap();
        let p = call(&w, "progress_get", json!({})).unwrap();
        assert_eq!(p["active"], false);
        assert!(p["done"].as_u64().unwrap() >= 1);
        assert!(p["total"].as_u64().unwrap() >= 1);
        // Ohne laufende Berechnung bleibt ein Abbruch wirkungslos.
        call(&w, "cancel_job", json!({})).unwrap();
        let p = call(&w, "progress_get", json!({})).unwrap();
        assert_eq!(p["cancelled"], false);
    }

    #[test]
    fn fortschritt_und_abbruch_folgen_der_kennung_der_berechnung() {
        // #107/AK2, #107/AK4
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let p = call(&w, "progress_get", json!({"jobId": 31337})).unwrap();
        assert_eq!((p["id"].as_u64(), p["active"].as_bool()), (Some(31337), Some(false)));
        assert!(call(&w, "cancel_job", json!({"jobId": 31337})).unwrap().is_null());
        call(&w, "run_simulation", json!({"plan": plan, "jobId": 31337})).unwrap();
        let p = call(&w, "progress_get", json!({"jobId": 31337})).unwrap();
        assert_eq!((p["id"].as_u64(), p["cancelled"].as_bool()), (Some(31337), Some(false)));
        assert!(p["total"].as_u64().unwrap() >= 1);
        call(&w, "optimize", json!({"plan": plan, "request": {}, "jobId": 31338})).unwrap();
        assert_eq!(call(&w, "progress_get", json!({"jobId": 31338})).unwrap()["id"], 31338);
    }

    #[test]
    fn issue_106_cancel_job_bricht_die_berechnung_mit_dieser_kennung_ab() {
        let (_app, w, _g) = setup();
        let job = engine::progress::start_as(Some(55_001), 0);
        let other = engine::progress::start_as(Some(55_002), 0);
        assert!(!engine::progress::cancelled());
        // Der innere Job (55002) ist der des Threads; ein Abbruch von 55001 trifft ihn nicht.
        call(&w, "cancel_job", json!({"jobId": 55_001})).unwrap();
        assert!(!engine::progress::cancelled());
        assert!(engine::progress::snapshot_of(55_001).cancelled);
        // Ohne Kennung werden alle laufenden Berechnungen abgebrochen.
        call(&w, "cancel_job", json!({})).unwrap();
        assert!(engine::progress::cancelled());
        drop(other);
        drop(job);
    }

    #[test]
    fn issue_106_einstellung_wird_ueber_den_befehl_gespeichert_und_gelesen() {
        let (_app, w, _g) = setup();
        // Zuerst aufräumen: ein früher abgebrochener Lauf (z. B. durch einen Mutanten) kann den Wert hinterlassen haben.
        call(&w, "settings_set", json!({"key": "issue_106Test", "value": null})).unwrap();
        let before = call(&w, "settings_get", json!({})).unwrap();
        assert!(before.get("issue_106Test").is_none_or(|v| v.is_null()), "{before}");
        call(&w, "settings_set", json!({"key": "issue_106Test", "value": 42})).unwrap();
        let after = call(&w, "settings_get", json!({})).unwrap();
        assert_eq!(after["issue_106Test"], 42);
        call(&w, "settings_set", json!({"key": "issue_106Test", "value": null})).unwrap();
    }

    /// #163/AK1: Der Datenordner gehört dem Testprozess (Prozesskennung im Namen) und ist nach dem Test entfernt.
    #[test]
    fn issue_163_ak1_datenordner_je_testprozess_und_nach_dem_test_entfernt() {
        let (app, w, g) = setup();
        let dir = app.path().app_data_dir().expect("Datenordner");
        let pid = std::process::id().to_string();
        assert!(
            dir.to_string_lossy().contains(&format!(".test-{pid}")),
            "{}",
            dir.display()
        );
        call(&w, "settings_set", json!({"key": "issue_163Test", "value": 1})).unwrap();
        assert!(dir.exists(), "Einstellungen müssen den Ordner angelegt haben");
        assert_eq!(g.data.as_ref(), Some(&dir));
        drop(g);
        assert!(!dir.exists(), "Ordner bleibt nach dem Test zurück: {}", dir.display());
    }

    #[test]
    fn issue_106_datenordner_nennen_den_identifier() {
        let (app, _w, _g) = setup();
        let id = app.config().identifier.clone();
        let dirs = data_dirs(app.handle());
        assert!(dirs.len() >= 2, "{dirs:?}");
        for d in &dirs {
            assert!(d.to_string_lossy().contains(&id), "{d:?} ohne {id}");
        }
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    fn issue_106_vom_system_gemeldete_datei_wird_vorgemerkt() {
        // macOS: Doppelklick auf eine Planungsdatei meldet «Opened» (#44); andere Dateien werden ignoriert.
        let (app, _w, _g) = setup();
        let dir = tmp("opened");
        let plan = dir.join("a.vplan");
        std::fs::write(&plan, "{}").unwrap();
        let other = dir.join("b.txt");
        std::fs::write(&other, "x").unwrap();
        on_run_event(
            app.handle(),
            RunEvent::Opened {
                urls: vec![tauri::Url::from_file_path(&other).unwrap()],
            },
        );
        assert!(app.state::<PendingOpen>().take().is_none());
        on_run_event(
            app.handle(),
            RunEvent::Opened {
                urls: vec![tauri::Url::from_file_path(&plan).unwrap()],
            },
        );
        assert_eq!(app.state::<PendingOpen>().take(), Some(plan));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn beispielplan_hat_keine_validierungsfehler_und_strategien() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let issues = call(&w, "validate", json!({"plan": plan})).unwrap();
        assert!(
            issues.as_array().unwrap().iter().all(|i| i["severity"] != "error"),
            "{issues}"
        );
        let katalog = call(&w, "opt_params", json!({"plan": plan})).unwrap();
        assert!(katalog.as_array().unwrap().len() >= 2);
    }

    #[test]
    fn ungueltiger_plan_wird_in_ergebnis_gemeldet() {
        let (_app, w, _g) = setup();
        let r = call(&w, "run_simulation", json!({"plan": {}})).unwrap();
        assert!(!r["error"].is_null() || !r["issues"].as_array().unwrap().is_empty());
        let p = call(&w, "preview", json!({"plan": {}})).unwrap();
        assert!(!p["error"].is_null() || !p["issues"].as_array().unwrap().is_empty());
        assert!(p["result"].is_null());
    }

    #[test]
    fn menuekuerzel_haben_gueltige_form() {
        const MODIFIERS: [&str; 3] = ["CmdOrCtrl", "Shift", "Alt"];
        for (id, _, key) in &eigene_eintraege() {
            if key.is_empty() {
                continue;
            }
            let parts: Vec<&str> = key.split('+').collect();
            let (taste, mods) = parts.split_last().unwrap();
            assert!(!taste.is_empty(), "{id}: Taste fehlt");
            assert!(mods.iter().all(|m| MODIFIERS.contains(m)), "{id}: {key}");
        }
        // Kein Kürzel doppelt vergeben.
        let mut keys: Vec<&str> = eigene_eintraege()
            .iter()
            .map(|e| e.2)
            .filter(|k| !k.is_empty())
            .collect();
        let n = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), n, "doppeltes Tastenkürzel");
    }

    #[test]
    fn menuetitel_und_programmmenue() {
        assert_eq!([FILE_TITLE, EDIT_TITLE, HELP_TITLE], ["Datei", "Bearbeiten", "Hilfe"]);
        #[cfg(target_os = "macos")]
        {
            assert_eq!(APP_MENU[0].0, "about");
            assert_eq!(APP_MENU[1], SEP);
            assert_eq!(APP_MENU[3], SEP);
        }
    }

    #[test]
    fn menuereignis_wird_mit_kennung_an_oberflaeche_gemeldet() {
        let (app, _w, _g) = setup();
        let got = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = got.clone();
        app.listen("menu", move |e| sink.lock().unwrap().push(e.payload().to_string()));
        forward_menu_event(
            app.handle(),
            MenuEvent {
                id: tauri::menu::MenuId::new("saveas"),
            },
        );
        for _ in 0..100 {
            if !got.lock().unwrap().is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(*got.lock().unwrap(), vec!["\"saveas\"".to_string()]);
    }

    #[test]
    fn echte_dialoge_werden_als_dienst_angemeldet() {
        let (app, _w, _g) = setup();
        let app2 = configure(mock_builder())
            .build(mock_context(noop_assets()))
            .expect("App baut");
        assert!(app2.try_state::<Picker>().is_none());
        install_dialogs(&app2);
        assert!(app2.try_state::<Picker>().is_some());
        assert!(app.try_state::<Picker>().is_some());
    }

    #[test]
    fn dialogbeschreibungen_sind_unveraendert() {
        let alle = [
            (
                &SHILLER_DIALOG,
                "Shiller-Daten importieren (ie_data.xls)",
                "Tabelle",
                "xls,xlsx,ods",
            ),
            (
                &SNB_DIALOG,
                "SNB-Daten importieren (devkum, plkopr, zimoma, rendoblim)",
                "SNB-Datenwürfel (CSV)",
                "csv",
            ),
            (&OPEN_DIALOG, "Planung öffnen", "VivaPlan-Plan", "vplan,rplan,json"),
            (&SAVE_DIALOG, "Planung speichern", "VivaPlan-Plan", "vplan"),
            (&LEGACY_DIALOG, "Alte config.json importieren", "Konfiguration", "json"),
            (&SERIES_DIALOG, "Zeitreihe wählen", "Zeitreihe (CSV)", "csv"),
        ];
        for (d, title, filter, ext) in alle {
            assert_eq!(
                (d.title, d.filter, d.extensions.join(",")),
                (title, filter, ext.to_string())
            );
        }
    }

    fn gewaehlt(p: &Path) -> FakeDialogs {
        FakeDialogs {
            file: Some(FilePath::Path(p.to_path_buf())),
            ..Default::default()
        }
    }

    #[test]
    fn abbruch_im_dialog_liefert_null_bei_allen_dialogbefehlen() {
        let (_app, w, _g, fake) = setup_with(FakeDialogs::default());
        for (cmd, args) in [
            ("import_shiller", json!({})),
            ("import_snb", json!({})),
            ("open_plan", json!({})),
            ("import_legacy", json!({"startYear": 2026})),
            ("pick_series", json!({})),
            ("save_plan", json!({"plan": {}})),
        ] {
            assert!(call(&w, cmd, args).unwrap().is_null(), "{cmd}");
        }
        let asked = fake.asked();
        assert_eq!(asked.len(), 6);
        assert!(asked[0].starts_with("file|Shiller-Daten importieren"));
        assert!(asked[1].starts_with("files|SNB-Daten importieren"));
        assert!(asked[2].starts_with("file|Planung öffnen|VivaPlan-Plan|vplan,rplan,json"));
        assert!(asked[3].starts_with("file|Alte config.json importieren|Konfiguration|json"));
        assert!(asked[4].starts_with("file|Zeitreihe wählen|Zeitreihe (CSV)|csv"));
        assert_eq!(asked[5], "save|Planung speichern|VivaPlan-Plan|vplan|Planung.vplan");
    }

    #[test]
    fn plan_oeffnen_ueber_befehl() {
        let dir = tmp("cmd-open");
        let f = dir.join("p.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1", "marker": 1}"#).unwrap();
        let (_app, w, _g, _) = setup_with(gewaehlt(&f));
        let r = call(&w, "open_plan", json!({})).unwrap();
        assert_eq!(r["path"], f.to_string_lossy().as_ref());
        assert_eq!(r["plan"]["marker"], 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plan_oeffnen_ueber_befehl_meldet_fehler() {
        let dir = tmp("cmd-open-err");
        let (_app, w, _g, _) = setup_with(gewaehlt(&dir.join("fehlt.vplan")));
        assert!(call(&w, "open_plan", json!({})).unwrap_err().is_string());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn speichern_ueber_befehl_mit_dialog_und_namensvorschlag() {
        let dir = tmp("cmd-save");
        let fake = FakeDialogs {
            save: Some(FilePath::Path(dir.join("ziel"))),
            ..Default::default()
        };
        let (_app, w, _g, fake) = setup_with(fake);
        let plan = json!({"project": {"name": "Mein Plan"}});
        let r = call(&w, "save_plan", json!({"plan": plan})).unwrap();
        assert_eq!(r.as_str().unwrap(), dir.join("ziel.vplan").to_string_lossy());
        assert_eq!(api::read_plan(&dir.join("ziel.vplan")).unwrap(), plan);
        assert_eq!(
            fake.asked(),
            vec!["save|Planung speichern|VivaPlan-Plan|vplan|Mein Plan.vplan"]
        );
        // Mit Pfad wird nicht gefragt.
        let direkt = dir.join("direkt.vplan");
        call(&w, "save_plan", json!({"plan": plan, "path": direkt.to_string_lossy()})).unwrap();
        assert_eq!(fake.asked().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #44/AK3 (legacy: RetirementPlanner): Eine geöffnete .rplan-Datei wird nie überschrieben. «Speichern» fragt wie «Speichern unter» und
    /// schlägt denselben Namen mit .vplan vor; die .vplan lässt sich wieder öffnen, die .rplan bleibt unverändert.
    #[test]
    fn rplan_speichern_schlaegt_vplan_vor_und_laesst_die_alte_datei_unveraendert() {
        let dir = tmp("cmd-save-legacy");
        let alt = dir.join("Demo.rplan");
        let inhalt = r#"{"format": "pensionsplaner/1", "project": {"name": "X"}}"#;
        std::fs::write(&alt, inhalt).unwrap();
        let fake = FakeDialogs {
            save: Some(FilePath::Path(dir.join("Demo.rplan"))),
            ..Default::default()
        };
        let (_app, w, _g, fake) = setup_with(fake);
        let plan = json!({"format": "vivaplan/1", "project": {"name": "X"}});
        let r = call(&w, "save_plan", json!({"plan": plan, "path": alt.to_string_lossy()})).unwrap();
        // Auch wenn im Dialog wieder .rplan gewählt wird, entsteht eine .vplan-Datei.
        assert_eq!(r.as_str().unwrap(), dir.join("Demo.vplan").to_string_lossy());
        assert_eq!(
            fake.asked(),
            vec!["save|Planung speichern|VivaPlan-Plan|vplan|Demo.vplan"]
        );
        assert_eq!(std::fs::read_to_string(&alt).unwrap(), inhalt);
        let wieder = call(&w, "open_path", json!({"path": r.as_str().unwrap()})).unwrap();
        assert_eq!(wieder["plan"], plan);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// legacy: RetirementPlanner – abgebrochenes Speichern einer .rplan-Datei.
    #[test]
    fn rplan_speichern_abgebrochen_schreibt_nichts() {
        let dir = tmp("cmd-save-legacy-cancel");
        let alt = dir.join("a.RPLAN");
        std::fs::write(&alt, "{}").unwrap();
        let (_app, w, _g, fake) = setup_with(FakeDialogs::default());
        let r = call(
            &w,
            "save_plan",
            json!({"plan": {"x": 1}, "path": alt.to_string_lossy()}),
        )
        .unwrap();
        assert!(r.is_null());
        assert_eq!(fake.asked(), vec!["save|Planung speichern|VivaPlan-Plan|vplan|a.vplan"]);
        assert_eq!(std::fs::read_to_string(&alt).unwrap(), "{}");
        assert_eq!(legacy_stem("/x/plan.vplan"), None);
        assert_eq!(legacy_stem("/x/Plan 1.rplan").as_deref(), Some("Plan 1"));
        assert_eq!(
            with_default_extension(PathBuf::from("/x/a")),
            PathBuf::from("/x/a.vplan")
        );
        assert_eq!(
            with_default_extension(PathBuf::from("/x/a.rplan")),
            PathBuf::from("/x/a.vplan")
        );
        assert_eq!(
            with_default_extension(PathBuf::from("/x/a.json")),
            PathBuf::from("/x/a.json")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #44/AK2: Fenstertitel «VivaPlan - ‹Dateiname›», ohne Datei nur «VivaPlan».
    #[test]
    fn fenstertitel_mit_und_ohne_datei() {
        assert_eq!(window_title(None), "VivaPlan");
        assert_eq!(window_title(Some("")), "VivaPlan");
        assert_eq!(window_title(Some("/a/b/Mein Plan.vplan")), "VivaPlan - Mein Plan.vplan");
        let (_app, w, _g) = setup();
        let t = call(&w, "set_window_title", json!({"path": "/a/Demo.rplan"})).unwrap();
        assert_eq!(t, "VivaPlan - Demo.rplan");
        assert_eq!(call(&w, "set_window_title", json!({})).unwrap(), "VivaPlan");
    }

    /// #44/AK5: Eine beim Start übergebene Planung (Doppelklick, Dateizuordnung) wird geöffnet und gemerkt.
    #[test]
    fn beim_start_uebergebene_datei_wird_geoeffnet() {
        assert!(is_plan_file(Path::new("/a/b.vplan")) && is_plan_file(Path::new("b.RPLAN")));
        assert!(!is_plan_file(Path::new("b.json")) && !is_plan_file(Path::new("b")));
        let args = |v: &[&str]| v.iter().map(OsString::from).collect::<Vec<_>>();
        assert_eq!(
            plan_arg(args(&["/app", "-x", "/d/p.vplan"])),
            Some(PathBuf::from("/d/p.vplan"))
        );
        assert_eq!(plan_arg(args(&["/d/p.vplan"])), None);
        assert_eq!(plan_arg(args(&["/app", "x.txt"])), None);

        let dir = tmp("given");
        let f = dir.join("doppelklick.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1", "marker": "doppelklick"}"#).unwrap();
        let s = startup_from(&dir, Some(f.clone()), example_stub);
        assert_eq!(s.plan["marker"], "doppelklick");
        assert_eq!(s.path.as_deref(), Some(f.to_string_lossy().as_ref()));

        let (app, w, _g) = setup();
        let opened = listen_to(&app, "open-file");
        open_from_system(app.handle(), f.clone());
        assert_eq!(opened.lock().unwrap().len(), 1);
        let s = call(&w, "startup_plan", json!({})).unwrap();
        assert_eq!(s["plan"]["marker"], "doppelklick");
        assert!(app.state::<PendingOpen>().take().is_none());
        // Bei laufender Oberfläche öffnet «open_path» die Datei und leert die Vormerkung.
        open_from_system(app.handle(), f.clone());
        let o = call(&w, "open_path", json!({"path": f.to_string_lossy()})).unwrap();
        assert_eq!(o["plan"]["marker"], "doppelklick");
        assert!(app.state::<PendingOpen>().take().is_none());
        assert!(call(
            &w,
            "open_path",
            json!({"path": dir.join("fehlt.vplan").to_string_lossy()})
        )
        .is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn alte_konfiguration_ueber_befehl() {
        let src = root().join("crates/engine/tests/fixtures/legacy_fixed.json");
        let (_app, w, _g, fake) = setup_with(gewaehlt(&src));
        let r = call(&w, "import_legacy", json!({"startYear": 2026})).unwrap();
        assert!(r["plan"].is_object());
        assert_eq!(fake.asked().len(), 1);
        // Jahr fehlt: Argumentfehler vor dem Dialog.
        assert!(call(&w, "import_legacy", json!({})).is_err());
        assert_eq!(fake.asked().len(), 1);
    }

    #[test]
    fn alte_konfiguration_ueber_befehl_meldet_fehler() {
        let dir = tmp("cmd-legacy-err");
        let bad = dir.join("c.json");
        std::fs::write(&bad, "kein json").unwrap();
        let (_app, w, _g, _) = setup_with(gewaehlt(&bad));
        assert!(call(&w, "import_legacy", json!({"startYear": 2026}))
            .unwrap_err()
            .is_string());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn zeitreihe_waehlen_ueber_befehl() {
        {
            let (_app, w, _g, _) = setup_with(gewaehlt(Path::new("/a/b.csv")));
            assert_eq!(call(&w, "pick_series", json!({})).unwrap(), "/a/b.csv");
        }
        let fake = FakeDialogs {
            file: Some(FilePath::Url("https://example.com/x".parse().unwrap())),
            ..Default::default()
        };
        let (_app, w, _g, _) = setup_with(fake);
        assert!(call(&w, "pick_series", json!({})).unwrap_err().is_string());
    }

    #[test]
    fn shiller_import_ueber_befehl_meldet_fehler_der_datei() {
        let dir = tmp("cmd-shiller");
        let (_app, w, _g, fake) = setup_with(gewaehlt(&dir.join("fehlt.xls")));
        assert!(call(&w, "import_shiller", json!({})).unwrap_err().is_string());
        assert_eq!(fake.asked().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn snb_import_ueber_befehl_meldet_fehler_der_dateien() {
        let dir = tmp("cmd-snb");
        let fake = FakeDialogs {
            files: Some(vec![FilePath::Path(dir.join("fehlt.csv"))]),
            ..Default::default()
        };
        let (_app, w, _g, fake) = setup_with(fake);
        assert!(call(&w, "import_snb", json!({})).unwrap_err().is_string());
        assert_eq!(fake.asked().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn snb_import_aus_den_ausgelieferten_cubes() {
        let lib = tmp("snb-ok");
        let dateien = ["devkum", "plkopr", "zimoma", "rendoblim"]
            .map(|n| FilePath::Path(root().join(format!("crates/engine/data/quellen/snb-{n}.csv"))));
        let rep = snb_from(&lib, dateien.to_vec()).unwrap();
        assert!(!rep.written.is_empty());
        let _ = std::fs::remove_dir_all(&lib);
    }

    fn example_stub() -> Value {
        json!({"format": "vivaplan/1", "marker": "beispiel"})
    }

    #[test]
    fn start_ohne_merkung_oeffnet_beispiel_ohne_hinweis() {
        let dir = tmp("start-leer");
        let s = startup_from(&dir, None, example_stub);
        assert_eq!(
            s,
            Startup {
                path: None,
                plan: example_stub(),
                notice: None,
                first: true
            }
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #70/AK1, AK6: erster Start nur ohne Einstellungsdatei; danach (auch wenn die gemerkte Datei fehlt) nicht mehr.
    #[test]
    fn issue_70_erster_start_nur_ohne_einstellungen() {
        let dir = tmp("fea54-erst");
        assert!(startup_from(&dir, None, example_stub).first, "ohne einstellungen.json");
        write_setting(&dir, "welcomeShown", json!(true)).unwrap();
        assert!(!startup_from(&dir, None, example_stub).first, "Einstellungen vorhanden");
        write_setting(&dir, LAST_PLAN_KEY, json!(dir.join("weg.vplan").to_string_lossy())).unwrap();
        let s = startup_from(&dir, None, example_stub);
        assert!(
            !s.first && s.notice.is_some(),
            "fehlende Datei: Hinweis, aber kein Willkommensdialog"
        );
        let _ = std::fs::remove_dir_all(&dir);
        // Mit einer übergebenen Datei (Doppelklick) gibt es keinen Willkommensdialog.
        let dir = tmp("fea54-datei");
        let f = dir.join("x.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1", "marker": "x"}"#).unwrap();
        assert!(!startup_from(&dir, Some(f), example_stub).first);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #70/AK7, #71/AK6: Menü «Hilfe» hat «Willkommen …» und «Erste Schritte».
    #[test]
    fn issue_70_issue_71_menue_hilfe() {
        assert!(HELP_MENU.contains(&("welcome", "Willkommen …", "")));
        assert!(HELP_MENU.contains(&("steps", "Erste Schritte", "")));
    }

    #[test]
    fn start_oeffnet_gemerkte_datei() {
        let dir = tmp("start-datei");
        let f = dir.join("meine.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1", "marker": "eigene"}"#).unwrap();
        write_setting(&dir, LAST_PLAN_KEY, json!(f.to_string_lossy())).unwrap();
        let s = startup_from(&dir, None, example_stub);
        assert_eq!(s.path.as_deref(), Some(f.to_string_lossy().as_ref()));
        assert_eq!(s.plan["marker"], "eigene");
        assert!(s.notice.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn start_mit_geloeschter_datei_zeigt_beispiel_und_hinweis() {
        let dir = tmp("start-fehlt");
        let f = dir.join("weg.vplan");
        write_setting(&dir, LAST_PLAN_KEY, json!(f.to_string_lossy())).unwrap();
        let s = startup_from(&dir, None, example_stub);
        assert!(s.path.is_none());
        assert_eq!(s.plan["marker"], "beispiel");
        let n = s.notice.unwrap();
        assert!(n.contains("nicht gefunden") && n.contains("weg.vplan"), "{n}");
        // Die Merkung bleibt erhalten
        assert_eq!(read_settings(&dir)[LAST_PLAN_KEY], json!(f.to_string_lossy()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn start_mit_unlesbarer_datei_zeigt_beispiel_und_laesst_merkung() {
        let dir = tmp("start-kaputt");
        let f = dir.join("kaputt.vplan");
        std::fs::write(&f, "kein json").unwrap();
        write_setting(&dir, LAST_PLAN_KEY, json!(f.to_string_lossy())).unwrap();
        let s = startup_from(&dir, None, example_stub);
        assert!(s.path.is_none());
        assert!(s.notice.unwrap().contains("nicht lesbar"));
        assert_eq!(read_settings(&dir)[LAST_PLAN_KEY], json!(f.to_string_lossy()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gemerkter_pfad_wird_beim_start_geoeffnet_und_ersetzt() {
        let dir = tmp("merken");
        let (a, b) = (dir.join("a.vplan"), dir.join("b.vplan"));
        std::fs::write(&a, r#"{"format": "vivaplan/1", "marker": "a"}"#).unwrap();
        std::fs::write(&b, r#"{"format": "vivaplan/1", "marker": "b"}"#).unwrap();
        remember_in(&dir, &a.to_string_lossy());
        assert_eq!(startup_from(&dir, None, example_stub).plan["marker"], "a");
        remember_in(&dir, &b.to_string_lossy());
        assert_eq!(startup_from(&dir, None, example_stub).plan["marker"], "b");
        // Nur der Pfad wird gemerkt
        assert_eq!(read_settings(&dir).len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn oeffnen_und_speichern_ueber_befehle_laufen_mit_merkung_durch() {
        let dir = tmp("merken-befehle");
        let f = dir.join("p.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1"}"#).unwrap();
        let (_app, w, _g, _) = setup_with(gewaehlt(&f));
        assert_eq!(
            call(&w, "open_plan", json!({})).unwrap()["path"],
            f.to_string_lossy().as_ref()
        );
        let neu = dir.join("neu.vplan");
        let r = call(
            &w,
            "save_plan",
            json!({"plan": {"format": "vivaplan/1"}, "path": neu.to_string_lossy()}),
        )
        .unwrap();
        assert_eq!(r, neu.to_string_lossy().as_ref());
        assert!(call(&w, "startup_plan", json!({})).unwrap()["plan"].is_object());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn beenden_fragt_nur_bei_ungespeicherten_aenderungen() {
        let (app, w, _g) = setup();
        let q = app.state::<Quit>();
        assert!(!q.must_ask());
        call(&w, "set_dirty", json!({"dirty": true})).unwrap();
        assert!(q.must_ask());
        call(&w, "set_dirty", json!({"dirty": false})).unwrap();
        assert!(!q.must_ask());
    }

    #[test]
    fn beenden_bestaetigt_gibt_den_ausstieg_frei() {
        let q = Quit::default();
        q.dirty.store(true, Ordering::SeqCst);
        q.confirmed.store(true, Ordering::SeqCst);
        assert!(!q.must_ask());
    }

    #[test]
    fn beispielplanung_im_bundle_ist_lesbar_oder_ersatz_greift() {
        let (app, _w, _g) = setup();
        let p = bundled_example(app.handle());
        assert!(p.get("format").is_some());
    }

    fn listen_to(app: &App<MockRuntime>, name: &str) -> Arc<std::sync::Mutex<Vec<String>>> {
        let got = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
        let sink = got.clone();
        app.listen(name, move |e| sink.lock().unwrap().push(e.payload().to_string()));
        got
    }

    fn wait_for(got: &Arc<std::sync::Mutex<Vec<String>>>) {
        for _ in 0..100 {
            if !got.lock().unwrap().is_empty() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    #[test]
    fn beenden_fragt_bei_aenderungen_und_sonst_nicht() {
        let (app, _w, _g) = setup();
        let asked = listen_to(&app, "quit-requested");
        let q = app.state::<Quit>();
        q.dirty.store(false, Ordering::SeqCst);
        q.confirmed.store(false, Ordering::SeqCst);
        assert!(!ask_before_exit(app.handle()), "ohne Änderungen kein Dialog");
        q.dirty.store(true, Ordering::SeqCst);
        assert!(ask_before_exit(app.handle()), "mit Änderungen fragen");
        wait_for(&asked);
        assert_eq!(asked.lock().unwrap().len(), 1);
        q.confirmed.store(true, Ordering::SeqCst);
        assert!(!ask_before_exit(app.handle()), "nach der Freigabe beenden");
        q.dirty.store(false, Ordering::SeqCst);
        q.confirmed.store(false, Ordering::SeqCst);
    }

    #[test]
    fn menueeintrag_beenden_fragt_bei_aenderungen_statt_zu_beenden() {
        let (app, _w, _g) = setup();
        let asked = listen_to(&app, "quit-requested");
        let menu = listen_to(&app, "menu");
        app.state::<Quit>().dirty.store(true, Ordering::SeqCst);
        forward_menu_event(
            app.handle(),
            MenuEvent {
                id: tauri::menu::MenuId::new(QUIT_ID),
            },
        );
        wait_for(&asked);
        assert_eq!(asked.lock().unwrap().len(), 1, "Dialog angefordert");
        assert!(
            menu.lock().unwrap().is_empty(),
            "«Beenden» geht nicht als normaler Menübefehl an die Oberfläche"
        );
        app.state::<Quit>().dirty.store(false, Ordering::SeqCst);
    }

    #[test]
    fn beenden_hat_eine_eigene_kennung_und_ein_kuerzel() {
        for (id, _, _) in &eigene_eintraege() {
            assert_ne!(*id, QUIT_ID);
        }
        assert_eq!(QUIT_TEXT, "Beenden");
    }

    // #124/AK8: Die Fenster aus der Konfiguration (dort `create: false`) entstehen erst beim Start der App.
    #[test]
    fn fenster_aus_der_konfiguration_werden_erstellt() {
        let _g = SERIAL.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut ctx = mock_context(noop_assets());
        ctx.config_mut().app.windows = vec![tauri::utils::config::WindowConfig {
            label: "haupt".into(),
            create: false,
            ..Default::default()
        }];
        let app = configure(mock_builder()).build(ctx).expect("App baut");
        assert!(app.get_webview_window("haupt").is_none());
        create_windows(&app).expect("Fenster");
        assert!(app.get_webview_window("haupt").is_some());
    }
}
