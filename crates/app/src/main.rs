//! Desktop app: window with the user interface from `crates/app/ui`, commands for the engine and for files.
//! Files are chosen only through dialogs and read or written locally.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;
mod migrate;

// #116: memory limit per test process (engine::test_alloc).
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
use tauri::menu::{CheckMenuItemBuilder, Menu, MenuEvent, MenuItemBuilder, Submenu, SubmenuBuilder};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager, RunEvent, Runtime, State, WindowEvent};
use tauri_plugin_dialog::{DialogExt, FilePath};

fn home() -> PathBuf {
    home_from(std::env::var_os("HOME"), std::env::var_os("USERPROFILE"))
}

/// Home directory from `HOME`, otherwise `USERPROFILE` (Windows); else empty.
fn home_from(home: Option<OsString>, user_profile: Option<OsString>) -> PathBuf {
    home.or(user_profile).map(PathBuf::from).unwrap_or_default()
}

/// Directory of the project file (for relative paths to time series).
fn base_dir(plan_path: Option<&str>) -> PathBuf {
    plan_path
        .and_then(|p| Path::new(p).parent().map(Path::to_path_buf))
        .unwrap_or_else(home)
}

/// #171: Error of a command for the user interface: a message of the engine (`{code, params, text}`, the user
/// interface translates it by the code) or a text of the operating system or of Tauri.
#[derive(Debug, PartialEq, Serialize)]
#[serde(untagged)]
enum CmdError {
    Message(engine::message::Message),
    Text(String),
}

#[cfg(test)]
impl CmdError {
    fn text(&self) -> &str {
        match self {
            CmdError::Message(m) => m.text(),
            CmdError::Text(s) => s,
        }
    }
}

impl From<String> for CmdError {
    fn from(s: String) -> Self {
        CmdError::Text(s)
    }
}

impl From<engine::Error> for CmdError {
    fn from(e: engine::Error) -> Self {
        CmdError::Message(e.message())
    }
}

impl From<engine::message::Message> for CmdError {
    fn from(m: engine::message::Message) -> Self {
        CmdError::Message(m)
    }
}

/// Folder of the data library (in the app's data directory, e.g. on macOS
/// `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`).
fn library_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    ensure_library_dir(&data)
}

/// Creates the subfolder «Zeitreihen» in the data directory (if needed) and returns it.
fn ensure_library_dir(data_dir: &Path) -> Result<PathBuf, String> {
    let dir = data_dir.join("Zeitreihen"); // i18n: allow (folder name)
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    Ok(dir)
}

/// File with the user interface settings in the data directory (settings only, no plan data).
const SETTINGS_FILE: &str = "einstellungen.json"; // i18n: allow (file name)

/// Reads the settings; a missing or unreadable file yields empty settings (the defaults apply).
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

/// Sets a setting and writes the file.
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

/// Setting holding the path of the plan last opened or saved (only the path, local).
const LAST_PLAN_KEY: &str = "lastPlan";
/// Name of the bundled neutral example plan (resource in the app bundle).
const EXAMPLE_RESOURCE: &str = "new.vplan";

/// Plan at startup: path (empty for the example plan), content and a notice for the user interface.
/// `first`: very first start (#70); the user interface then shows the welcome dialog.
#[derive(Serialize, Debug, PartialEq)]
struct Startup {
    path: Option<String>,
    plan: Value,
    notice: Option<Notice>,
    first: bool,
}

/// #170: notice as key of the text table with the file name; the user interface shows it in its language.
#[derive(Serialize, Debug, PartialEq)]
struct Notice {
    key: &'static str,
    file: String,
}

fn file_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// Determines the plan at startup. A file passed at startup (double-click, command line) takes precedence, otherwise
/// the remembered file applies. Without a remembered file the example plan applies (without notice); likewise if the
/// file is missing or unreadable (with notice). The remembered path itself stays unchanged.
/// #70: If there is no settings file yet and no file was passed, this is the very first start.
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
    let fallback = |key: &'static str| Startup {
        path: None,
        plan: example(),
        notice: Some(Notice {
            key,
            file: file_name(last),
        }),
        first: false,
    };
    if !Path::new(last).is_file() {
        return fallback("startup.notFound");
    }
    match api::read_plan(Path::new(last)) {
        Ok(plan) => Startup {
            path: Some(last.to_string()),
            plan,
            notice: None,
            first: false,
        },
        Err(_) => fallback("startup.unreadable"),
    }
}

/// Bundled example plan from the resources; otherwise the built-in one.
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

/// #44: File the app was started with (double-click in the Finder, file association, command line), until the
/// user interface asks for it at startup.
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

/// Extensions of plan files; `rplan` is the old extension (legacy: RetirementPlanner), it stays readable.
const PLAN_EXTENSIONS: [&str; 2] = ["vplan", "rplan"];

fn is_plan_file(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| PLAN_EXTENSIONS.iter().any(|x| e.eq_ignore_ascii_case(x)))
}

/// First plan file among the startup arguments (Windows and Linux pass the file this way).
fn plan_arg(args: impl IntoIterator<Item = OsString>) -> Option<PathBuf> {
    args.into_iter().skip(1).map(PathBuf::from).find(|p| is_plan_file(p))
}

/// Open a file from the operating system (macOS: double-click while the app is running or starting). If the user
/// interface is already running, it opens the file via the event «open-file»; at startup it fetches it with `startup_plan`.
#[cfg(any(test, target_os = "macos", target_os = "ios"))]
fn open_from_system<R: Runtime>(app: &AppHandle<R>, path: PathBuf) {
    app.state::<PendingOpen>().set(path.clone());
    let _ = app.emit("open-file", path.to_string_lossy());
}

/// Opens a file reported by the system (event «open-file»), without a dialog.
#[tauri::command]
async fn open_path<R: Runtime>(
    app: AppHandle<R>,
    pending: State<'_, PendingOpen>,
    path: String,
) -> Result<Opened, CmdError> {
    pending.take();
    let o = opened_from(FilePath::Path(PathBuf::from(path)))?;
    remember_plan(&app, &o.path);
    Ok(o)
}

/// Product name in the window title and in the menus.
const PRODUCT: &str = "VivaPlan";

/// Window title: «VivaPlan - ‹file name›», without a file just «VivaPlan».
fn window_title(path: Option<&str>) -> String {
    match path.filter(|p| !p.is_empty()) {
        Some(p) => format!("{PRODUCT} - {}", file_name(p)),
        None => PRODUCT.to_string(),
    }
}

/// Sets the window title after the file was opened or saved.
#[tauri::command]
fn set_window_title<R: Runtime>(window: tauri::WebviewWindow<R>, path: Option<String>) -> Result<String, String> {
    let title = window_title(path.as_deref());
    window.set_title(&title).map_err(|e| e.to_string())?;
    Ok(title)
}

/// Remembers the path of the plan for the next start; an error here does not disturb opening or saving.
fn remember_plan<R: Runtime>(app: &AppHandle<R>, path: &str) {
    if let Ok(data) = app.path().app_data_dir() {
        remember_in(&data, path);
    }
}

fn remember_in(data_dir: &Path, path: &str) {
    let _ = write_setting(data_dir, LAST_PLAN_KEY, Value::String(path.to_string()));
}

/// State for quitting: unsaved changes (reported by the user interface) and the permission to quit.
#[derive(Default)]
struct Quit {
    dirty: AtomicBool,
    confirmed: AtomicBool,
}

impl Quit {
    /// Quitting needs a confirmation as long as changes are unsaved and quitting has not been permitted.
    fn must_ask(&self) -> bool {
        self.dirty.load(Ordering::SeqCst) && !self.confirmed.load(Ordering::SeqCst)
    }
}

#[tauri::command]
fn set_dirty(quit: State<'_, Quit>, dirty: bool) {
    quit.dirty.store(dirty, Ordering::SeqCst);
}

/// Quits the app without further confirmation (the user interface has saved or discarded).
#[tauri::command]
fn quit_app<R: Runtime>(app: AppHandle<R>, quit: State<'_, Quit>) {
    quit.confirmed.store(true, Ordering::SeqCst);
    app.exit(0);
}

/// #141: smoke test of the shipped app (`--smoke`): the user interface starts, calculates the example plan and reports the
/// result with `smoke_done`. The app then exits with code 0 (ok) or 1 (error); without a report after
/// [`SMOKE_TIMEOUT`] with code 2.
#[derive(Default)]
struct Smoke(bool);

const SMOKE_FLAG: &str = "--smoke";
const SMOKE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(120);

fn smoke_requested(args: impl IntoIterator<Item = OsString>) -> bool {
    args.into_iter().skip(1).any(|a| a == SMOKE_FLAG)
}

/// Reports the result of the smoke test on the console and, if `VIVAPLAN_SMOKE_OUT` is set, in that file
/// (on Windows the app has no console).
fn smoke_report(line: &str) {
    println!("{line}");
    if let Some(out) = std::env::var_os("VIVAPLAN_SMOKE_OUT") {
        let _ = std::fs::write(out, format!("{line}\n"));
    }
}

/// Output and exit code of the smoke test.
fn smoke_outcome(ok: bool, message: &str) -> (String, i32) {
    (
        format!("VivaPlan-Rauchtest: {} – {message}", if ok { "ok" } else { "FEHLER" }), // i18n: allow (smoke test, only in the log)
        if ok { 0 } else { 1 },
    )
}

#[tauri::command]
fn smoke_done<R: Runtime>(app: AppHandle<R>, quit: State<'_, Quit>, ok: bool, message: String) {
    // Only in the smoke test (`main` sets the state); otherwise without effect.
    if !app.try_state::<Smoke>().is_some_and(|s| s.0) {
        return;
    }
    let (line, code) = smoke_outcome(ok, &message);
    smoke_report(&line);
    quit.confirmed.store(true, Ordering::SeqCst);
    app.exit(code);
}

/// Sends «quit-requested» to the user interface if there are unsaved changes; `true` means: stop quitting.
fn ask_before_exit<R: Runtime>(app: &AppHandle<R>) -> bool {
    let ask = app.state::<Quit>().must_ask();
    if ask {
        let _ = app.emit("quit-requested", ());
    }
    ask
}

/// Quitting through the system (e.g. Dock or logout): with unsaved changes stop and ask.
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

/// Close button of the window: with unsaved changes keep the window open and ask.
fn on_window_event<R: Runtime>(window: &tauri::Window<R>, event: &WindowEvent) {
    if let WindowEvent::CloseRequested { api, .. } = event {
        if ask_before_exit(window.app_handle()) {
            api.prevent_close();
        }
    }
}

/// Menu item «Beenden» (⌘Q): asks if there are unsaved changes, otherwise quits immediately.
fn quit_from_menu<R: Runtime>(app: &AppHandle<R>) {
    if !ask_before_exit(app) {
        app.exit(0);
    }
}

/// Description of a file dialog: title and file filters (name and extensions). Title and filter name are keys of the
/// text table (#170).
struct Dialog {
    title: &'static str,
    filter: &'static str,
    extensions: &'static [&'static str],
}

const SHILLER_DIALOG: Dialog = Dialog {
    title: "dialog.shiller",
    filter: "dialog.shillerFilter",
    extensions: &["xls", "xlsx", "ods"],
};
const SNB_DIALOG: Dialog = Dialog {
    title: "dialog.snb",
    filter: "dialog.snbFilter",
    extensions: &["csv"],
};
const OPEN_DIALOG: Dialog = Dialog {
    title: "dialog.open",
    filter: "dialog.planFilter",
    extensions: &["vplan", "rplan", "json"], // legacy: RetirementPlanner (rplan)
};
const SAVE_DIALOG: Dialog = Dialog {
    title: "dialog.save",
    filter: "dialog.planFilter",
    extensions: &["vplan"],
};
const LEGACY_DIALOG: Dialog = Dialog {
    title: "dialog.legacy",
    filter: "dialog.legacyFilter",
    extensions: &["json"],
};
const SERIES_DIALOG: Dialog = Dialog {
    title: "dialog.series",
    filter: "dialog.seriesFilter",
    extensions: &["csv"],
};

/// File picker of the operating system. Behind a trait so that tests can return fixed paths;
/// `None` means: dialog cancelled.
trait Dialogs: Send + Sync {
    fn pick_file(&self, dialog: &Dialog) -> Option<FilePath>;
    fn pick_files(&self, dialog: &Dialog) -> Option<Vec<FilePath>>;
    fn save_file(&self, dialog: &Dialog, file_name: &str) -> Option<FilePath>;
}

/// The shared managed file picker service (see `install_dialogs`).
type Picker = Arc<dyn Dialogs>;

/// Real dialogs via `tauri-plugin-dialog` (blocking, open windows of the operating system).
struct NativeDialogs<R: Runtime>(AppHandle<R>);

impl<R: Runtime> NativeDialogs<R> {
    fn builder(&self, dialog: &Dialog) -> tauri_plugin_dialog::FileDialogBuilder<R> {
        let lang = self.0.state::<i18n::UiLanguage>().language();
        self.0
            .dialog()
            .file()
            .set_title(i18n::tr(&lang, dialog.title))
            .add_filter(i18n::tr(&lang, dialog.filter), dialog.extensions)
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

/// Registers the real dialogs; the commands fetch them as `State<Picker>`.
fn install_dialogs<R: Runtime>(app: &tauri::App<R>) {
    let picker: Picker = Arc::new(NativeDialogs(app.handle().clone()));
    app.manage(picker);
}

fn path_string(p: FilePath) -> Result<PathBuf, String> {
    p.into_path().map_err(|e| e.to_string())
}

/// Suggested file name (without extension) from the project name, otherwise `default`.
fn plan_name<'a>(plan: &'a Value, default: &'a str) -> &'a str {
    plan.pointer("/project/name")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(default)
}

/// Adds the extension «vplan» if the chosen path has none; «.rplan» becomes «.vplan» so that an old file
/// is never overwritten.
fn with_default_extension(mut p: PathBuf) -> PathBuf {
    if p.extension().is_none() || legacy_stem(&p.to_string_lossy()).is_some() {
        p.set_extension("vplan");
    }
    p
}

/// legacy: RetirementPlanner – name (without extension) of an old .rplan file, otherwise `None`.
fn legacy_stem(path: &str) -> Option<String> {
    let p = Path::new(path);
    p.extension()
        .and_then(|e| e.to_str())
        .filter(|e| e.eq_ignore_ascii_case("rplan"))?;
    p.file_stem().map(|s| s.to_string_lossy().into_owned())
}

/// Base folder for data paths of an old config.json: the folder of the file or its parent folder,
/// depending on where the referenced file is.
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

/// Version and time of the build for the «About» window.
#[tauri::command]
fn app_info<R: Runtime>(app: AppHandle<R>) -> Value {
    serde_json::json!({
        "version": app.package_info().version.to_string(),
        "buildEpoch": env!("PP_BUILD_EPOCH").parse::<u64>().unwrap_or(0),
    })
}

/// Computing time per run, for the duration estimate of scenarios and optimization.
#[tauri::command]
async fn bench_plan<R: Runtime>(
    app: AppHandle<R>,
    plan: Value,
    path: Option<String>,
) -> Result<api::BenchOut, CmdError> {
    let lib = library_dir(&app).ok();
    api::bench(&plan, &base_dir(path.as_deref()), lib.as_deref()).map_err(CmdError::from)
}

/// State of a calculation (simulation or optimization); without an id the one started last.
#[tauri::command]
fn progress_get(job_id: Option<u64>) -> engine::progress::Progress {
    match job_id {
        Some(id) => engine::progress::snapshot_of(id),
        None => engine::progress::snapshot(),
    }
}

/// Cancels a calculation (with an id only that one, otherwise all running ones).
#[tauri::command]
fn cancel_job(job_id: Option<u64>) {
    match job_id {
        Some(id) => {
            engine::progress::cancel_job(id);
        }
        None => engine::progress::cancel(),
    }
}

/// Parameters of the strategies with default ranges and the values of the plan.
#[tauri::command]
fn opt_params(plan: Value) -> Vec<engine::optimize::CatalogStrategy> {
    api::opt_params(&plan)
}

#[tauri::command]
fn example_plan() -> Value {
    api::example_plan()
}

/// #138: templates of the property gains tax.
#[tauri::command]
fn gain_templates() -> Value {
    api::gain_templates()
}

#[tauri::command]
fn validate(plan: Value) -> Vec<Issue> {
    api::validate(&plan)
}

/// Brings an older plan up to date when loading (#52, #51).
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
) -> Result<engine::optimize::Report, CmdError> {
    let lib = library_dir(&app).ok();
    if let (Some(id), Some(obj)) = (job_id, request.as_object_mut()) {
        obj.insert("jobId".into(), id.into());
    }
    api::optimize(&plan, &request, &base_dir(path.as_deref()), lib.as_deref()).map_err(CmdError::from)
}

#[tauri::command]
fn series_info<R: Runtime>(
    app: AppHandle<R>,
    path: String,
    plan_path: Option<String>,
) -> Result<engine::library::SeriesInfo, CmdError> {
    let lib = library_dir(&app).ok();
    api::series_info(&path, &base_dir(plan_path.as_deref()), lib.as_deref()).map_err(CmdError::from)
}

#[tauri::command]
async fn library_list<R: Runtime>(app: AppHandle<R>) -> Result<api::LibraryOut, CmdError> {
    list_in(&library_dir(&app)?)
}

/// Content of the data library in folder `dir` (creates the shipped time series if needed).
fn list_in(dir: &Path) -> Result<api::LibraryOut, CmdError> {
    api::library_list(dir).map_err(CmdError::from)
}

/// Runs `then` only if something was chosen in the dialog (`pick` then returns `Some`).
fn picked<T, U>(
    pick: impl FnOnce() -> Option<T>,
    then: impl FnOnce(T) -> Result<U, CmdError>,
) -> Result<Option<U>, CmdError> {
    pick().map(then).transpose()
}

#[tauri::command]
async fn import_shiller<R: Runtime>(
    app: AppHandle<R>,
    dialogs: State<'_, Picker>,
) -> Result<Option<engine::library::ImportReport>, CmdError> {
    picked(
        || dialogs.pick_file(&SHILLER_DIALOG),
        |file| shiller_from(&library_dir(&app)?, file),
    )
}

fn shiller_from(library: &Path, file: FilePath) -> Result<engine::library::ImportReport, CmdError> {
    api::import_shiller(library, &path_string(file)?).map_err(CmdError::from)
}

#[tauri::command]
async fn import_snb<R: Runtime>(
    app: AppHandle<R>,
    dialogs: State<'_, Picker>,
) -> Result<Option<engine::library::ImportReport>, CmdError> {
    picked(
        || dialogs.pick_files(&SNB_DIALOG),
        |files| snb_from(&library_dir(&app)?, files),
    )
}

fn snb_from(library: &Path, files: Vec<FilePath>) -> Result<engine::library::ImportReport, CmdError> {
    let files = files.into_iter().map(path_string).collect::<Result<Vec<_>, _>>()?;
    api::import_snb(library, &files).map_err(CmdError::from)
}

#[tauri::command]
async fn open_plan<R: Runtime>(app: AppHandle<R>, dialogs: State<'_, Picker>) -> Result<Option<Opened>, CmdError> {
    let opened = picked(|| dialogs.pick_file(&OPEN_DIALOG), opened_from)?;
    if let Some(o) = &opened {
        remember_plan(&app, &o.path);
    }
    Ok(opened)
}

fn opened_from(file: FilePath) -> Result<Opened, CmdError> {
    let path = path_string(file)?;
    let plan = api::read_plan(&path)?;
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
) -> Result<Option<String>, CmdError> {
    let default = i18n::tr(&app.state::<i18n::UiLanguage>().language(), "dialog.planName");
    let saved = save_with(&plan, path, &default, |name| {
        dialogs.save_file(&SAVE_DIALOG, &format!("{name}.vplan"))
    })?;
    if let Some(p) = &saved {
        remember_plan(&app, p);
    }
    Ok(saved)
}

/// Saves under the given path, otherwise under the one chosen in the dialog (`pick`, with a suggested name).
/// #44: If the given path is an old .rplan file, the dialog asks like «Speichern unter» and suggests the same
/// name (with .vplan); the .rplan file stays unchanged.
fn save_with(
    plan: &Value,
    path: Option<String>,
    default_name: &str,
    pick: impl FnOnce(&str) -> Option<FilePath>,
) -> Result<Option<String>, CmdError> {
    let legacy = path.as_deref().and_then(legacy_stem);
    let target = match path {
        Some(p) if legacy.is_none() => Some(PathBuf::from(p)),
        _ => {
            let name = legacy.unwrap_or_else(|| plan_name(plan, default_name).to_string());
            picked(|| pick(&name), |file| Ok(with_default_extension(path_string(file)?)))?
        }
    };
    target.map(|t| write_to(&t, plan)).transpose()
}

fn write_to(target: &Path, plan: &Value) -> Result<String, CmdError> {
    api::write_plan(target, plan)?;
    Ok(target.to_string_lossy().into_owned())
}

#[tauri::command]
async fn import_legacy(dialogs: State<'_, Picker>, start_year: i32) -> Result<Option<api::ImportOut>, CmdError> {
    picked(
        || dialogs.pick_file(&LEGACY_DIALOG),
        |file| legacy_from(file, start_year),
    )
}

fn legacy_from(file: FilePath, start_year: i32) -> Result<api::ImportOut, CmdError> {
    let path = path_string(file)?;
    let text = std::fs::read_to_string(&path)
        .map_err(|e| engine::msg!(file_error, path = path.display().to_string(), detail = e.to_string()))?;
    // The old program ran in the project folder; data paths like ./data/MSCI.csv are relative to it.
    let base = legacy_base(&path, &text);
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    api::import_legacy(&text, start_year, &base, &name).map_err(CmdError::from)
}

fn series_choice(file: FilePath) -> Result<String, CmdError> {
    Ok(path_string(file)?.to_string_lossy().into_owned())
}

#[tauri::command]
async fn pick_series(dialogs: State<'_, Picker>) -> Result<Option<String>, CmdError> {
    picked(|| dialogs.pick_file(&SERIES_DIALOG), series_choice)
}

/// A menu entry: id, key of the text table and keyboard shortcut (empty = none).
type Entry = (&'static str, &'static str, &'static str);

/// Separator in the menu tables.
const SEP: Entry = ("", "", "");

const FILE_MENU: [Entry; 8] = [
    ("new", "menu.file.new", "CmdOrCtrl+N"),
    ("open", "menu.file.open", "CmdOrCtrl+O"),
    SEP,
    ("save", "menu.file.save", "CmdOrCtrl+S"),
    ("saveas", "menu.file.saveAs", "CmdOrCtrl+Shift+S"),
    SEP,
    ("import", "menu.file.import", "CmdOrCtrl+I"),
    ("run", "menu.file.run", "CmdOrCtrl+Enter"),
];

/// #25: application-wide settings (limits for AHV payout, pension fund payout, maximum amount pillar 3a).
/// #59: on macOS in the application menu, on Windows and Linux at the end of «Datei».
const SETTINGS: Entry = ("settings", "menu.settings", "CmdOrCtrl+,");

/// Entries of the menu «Datei» per system (without «Beenden», which follows as a system entry).
fn file_entries() -> Vec<Entry> {
    let tail: &[Entry] = if cfg!(target_os = "macos") {
        &[]
    } else {
        &[SEP, SETTINGS]
    };
    FILE_MENU.iter().chain(tail).copied().collect()
}

const EDIT_MENU: [Entry; 3] = [
    ("undo", "menu.edit.undo", "CmdOrCtrl+Z"),
    ("redo", "menu.edit.redo", "CmdOrCtrl+Shift+Z"),
    SEP,
];

/// Id of the menu item «Beenden»; the app handles it itself (confirmation with unsaved changes).
const QUIT_ID: &str = "quit";

/// Own rows of the application menu (macOS): «Über» and «Einstellungen…» with separators; the rest are
/// system entries.
#[cfg(target_os = "macos")]
const APP_MENU: [Entry; 4] = [("about", "menu.about", ""), SEP, SETTINGS, SEP];

const FILE_TITLE: &str = "menu.file";
const QUIT_TEXT: &str = "menu.quit";
const EDIT_TITLE: &str = "menu.edit";
const VIEW_TITLE: &str = "menu.view";
const HELP_TITLE: &str = "menu.help";

/// #70/#71: «Willkommen …» opens the welcome dialog, «Erste Schritte» shows the checklist.
const HELP_MENU: [Entry; 8] = [
    ("helpctx", "menu.help.topic", "F1"),
    ("manual", "menu.help.manual", "CmdOrCtrl+Shift+/"),
    ("help", "menu.help.shortcuts", "CmdOrCtrl+/"),
    SEP,
    ("welcome", "menu.help.welcome", ""),
    ("steps", "menu.help.steps", ""),
    SEP,
    ("about", "menu.about", ""),
];

/// Builds a submenu from a table; `title` is already translated, the entries are translated into `lang`.
fn submenu<'a, R: Runtime>(
    app: &'a AppHandle<R>,
    lang: &str,
    title: &str,
    entries: &[Entry],
) -> tauri::Result<SubmenuBuilder<'a, R, AppHandle<R>>> {
    let mut b = SubmenuBuilder::new(app, title);
    for (id, key, accel) in entries {
        if id.is_empty() {
            b = b.separator();
            continue;
        }
        let mut item = MenuItemBuilder::with_id(*id, i18n::tr(lang, key));
        if !accel.is_empty() {
            item = item.accelerator(*accel);
        }
        b = b.item(&item.build(app)?);
    }
    Ok(b)
}

/// Prefix of the ids of the entries in «Ansicht → Sprache»; the user interface handles them (`language-<choice>`).
const LANGUAGE_ID: &str = "language-";

/// #170/AK9: entries of «Ansicht → Sprache»: «wie System» and the available languages, each named in its own
/// language; the active choice is checked. (id, text, checked)
fn language_entries(lang: &str, choice: &str) -> Vec<(String, String, bool)> {
    std::iter::once((i18n::SYSTEM.to_string(), i18n::tr(lang, "language.system")))
        .chain(i18n::languages())
        .map(|(code, text)| (format!("{LANGUAGE_ID}{code}"), text, code == choice))
        .collect()
}

/// Submenu «Sprache» with check marks.
fn language_menu<R: Runtime>(app: &AppHandle<R>, lang: &str, choice: &str) -> tauri::Result<Submenu<R>> {
    let mut b = SubmenuBuilder::new(app, i18n::tr(lang, "menu.view.language"));
    for (i, (id, text, checked)) in language_entries(lang, choice).into_iter().enumerate() {
        if i == 1 {
            b = b.separator();
        }
        b = b.item(&CheckMenuItemBuilder::with_id(id, text).checked(checked).build(app)?);
    }
    b.build()
}

/// Menu with the usual keyboard shortcuts, in the language `lang`; `choice` is checked in «Ansicht → Sprache».
/// Own entries report to the user interface as event «menu» with their id; cut, copy and paste are handled by
/// the system.
fn menu<R: Runtime>(app: &AppHandle<R>, lang: &str, choice: &str) -> tauri::Result<Menu<R>> {
    let tr = |key: &str| i18n::tr(lang, key);
    let quit = MenuItemBuilder::with_id(QUIT_ID, tr(QUIT_TEXT))
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;
    #[cfg(not(target_os = "macos"))]
    let file = submenu(app, lang, &tr(FILE_TITLE), &file_entries())?
        .separator()
        .item(&quit)
        .build()?;
    #[cfg(target_os = "macos")]
    let file = submenu(app, lang, &tr(FILE_TITLE), &file_entries())?.build()?;
    let edit = submenu(app, lang, &tr(EDIT_TITLE), &EDIT_MENU)?
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;
    let view = SubmenuBuilder::new(app, tr(VIEW_TITLE))
        .item(&language_menu(app, lang, choice)?)
        .build()?;
    let help = submenu(app, lang, &tr(HELP_TITLE), &HELP_MENU)?.build()?;
    #[cfg(target_os = "macos")]
    {
        let app_menu = submenu(app, lang, PRODUCT, &APP_MENU)?
            .hide()
            .hide_others()
            .show_all()
            .separator()
            .item(&quit)
            .build()?;
        Menu::with_items(app, &[&app_menu, &file, &edit, &view, &help])
    }
    #[cfg(not(target_os = "macos"))]
    Menu::with_items(app, &[&file, &edit, &view, &help])
}

/// #170/AK2: language of the operating system, but only at the very first start (no settings file yet, see #70).
/// Later the user interface takes the languages of the web view for the choice «wie System».
#[tauri::command]
async fn first_start_language<R: Runtime>(app: AppHandle<R>) -> Result<Option<String>, String> {
    let data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(first_start_language_in(&data, i18n::system_language))
}

fn first_start_language_in(data_dir: &Path, system: impl FnOnce() -> String) -> Option<String> {
    (!data_dir.join(SETTINGS_FILE).exists()).then(system)
}

/// Marker: the native menu is installed (real application, not the tests with the mock runtime).
struct MenuInstalled;

/// #170/AK3, AK9: the user interface reports language and choice after every change (settings or menu
/// «Ansicht → Sprache»); file dialogs and menus follow at once.
#[tauri::command]
async fn set_menu_language<R: Runtime>(
    app: AppHandle<R>,
    ui: State<'_, i18n::UiLanguage>,
    language: String,
    choice: String,
) -> Result<(), String> {
    ui.set(&language, &choice);
    if app.try_state::<MenuInstalled>().is_some() {
        let (handle, lang, choice) = (app.clone(), ui.language(), ui.choice());
        app.run_on_main_thread(move || {
            if let Ok(m) = menu(&handle, &lang, &choice) {
                let _ = handle.set_menu(m);
            }
        })
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Language of the menus at the start, before the user interface reports its own: the saved choice, otherwise the
/// language of the operating system.
fn startup_language(data_dir: &Path) -> i18n::UiLanguage {
    let settings = read_settings(data_dir);
    i18n::startup(
        settings.get(i18n::SETTING).and_then(Value::as_str),
        i18n::system_language,
    )
}

/// Builds the application (plugins, menu, commands); separate from `main` so that it is testable with the mock runtime.
fn configure<R: Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    builder
        .plugin(tauri_plugin_dialog::init())
        .manage(Quit::default())
        .manage(PendingOpen::default())
        .manage(i18n::UiLanguage::new(
            i18n::FALLBACK.to_string(),
            i18n::SYSTEM.to_string(),
        ))
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
            first_start_language,
            set_menu_language,
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

/// Reports a menu click as event «menu» with the id of the entry to the user interface.
fn forward_menu_event<R: Runtime>(app: &AppHandle<R>, ev: MenuEvent) {
    if ev.id().0 == QUIT_ID {
        return quit_from_menu(app);
    }
    let _ = app.emit("menu", ev.id().0.as_str());
}

/// Sets the menu and menu events; needs the main thread of the operating system and is therefore only active in
/// the real application (not in the tests with the mock runtime).
fn install_menu<R: Runtime>(app: &mut tauri::App<R>) -> Result<(), Box<dyn std::error::Error>> {
    let ui = app.state::<i18n::UiLanguage>();
    if let Ok(data) = app.path().app_data_dir() {
        let start = startup_language(&data);
        ui.set(&start.language(), &start.choice());
    }
    let m = menu(app.handle(), &ui.language(), &ui.choice())?;
    app.set_menu(m)?;
    app.manage(MenuInstalled);
    app.on_menu_event(forward_menu_event);
    Ok(())
}

/// New data folders of the app whose last part is the identifier (for taking over the old app data).
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

/// Creates the windows described in the configuration file (there with `create: false`, so that the app data is
/// taken over before the web view accesses it for the first time).
fn create_windows<R: Runtime>(app: &tauri::App<R>) -> tauri::Result<()> {
    let smoke = app.try_state::<Smoke>().is_some_and(|s| s.0);
    for w in app.config().app.windows.clone() {
        let mut b = tauri::WebviewWindowBuilder::from_config(app.handle(), &w)?;
        if smoke {
            // #141: the user interface detects the smoke test by this variable (set before all scripts of the page).
            b = b.initialization_script("window.__VIVAPLAN_SMOKE__ = true;");
        }
        b.build()?;
    }
    Ok(())
}

/// Setup at the start of the real application: take over old app data (#44), windows, file dialogs and menu.
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
        .expect("VivaPlan konnte nicht gestartet werden"); // i18n: allow (start error, only in the log)
    if let Some(p) = given {
        app.state::<PendingOpen>().set(p);
    }
    if smoke {
        // #141: time limit in case the user interface never reports (e.g. the web view does not start)
        std::thread::spawn(|| {
            std::thread::sleep(SMOKE_TIMEOUT);
            smoke_report(&format!(
                "VivaPlan-Rauchtest: FEHLER – keine Meldung der Oberfläche nach {} s", // i18n: allow (smoke test, only in the log)
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

    /// Calculations share a global progress and cancel state; tests therefore run one after another.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// File dialog stand-in with fixed answers; remembers how it was asked.
    #[derive(Default)]
    struct FakeDialogs {
        file: Option<FilePath>,
        files: Option<Vec<FilePath>>,
        save: Option<FilePath>,
        asked: std::sync::Mutex<Vec<String>>,
    }

    impl FakeDialogs {
        fn note(&self, kind: &str, d: &Dialog, name: &str) {
            // #170: title and filter are keys of the text table; the native dialog shows them in German here
            let (title, filter) = (i18n::tr("de", d.title), i18n::tr("de", d.filter));
            let line = format!("{kind}|{title}|{filter}|{}|{name}", d.extensions.join(","));
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

    /// Holds the lock `SERIAL` and removes the data folder of the test process at the end of the test.
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

    /// #159: Every test process has its own data folder (settings, plan last opened). nextest starts
    /// every test in its own process; the lock `SERIAL` only applies within one process. With a shared
    /// folder a test running at the same time overwrote the settings (read, modify, write), and
    /// `issue_106_setting_is_saved_and_read_via_the_command` failed at random.
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

    /// #141: origin of the user interface as Tauri checks it: on Windows (WebView2) `http://tauri.localhost`, otherwise
    /// `tauri://localhost`. With the wrong origin Tauri rejects every command («… not allowed»).
    const APP_ORIGIN: &str = if cfg!(windows) {
        "http://tauri.localhost"
    } else {
        "tauri://localhost"
    };

    /// #171: Text of an error of a command: a text or a message of the engine (`{code, params, text}`).
    fn error_text(e: &Value) -> &str {
        match e.as_str() {
            Some(t) => t,
            None => {
                assert!(e["code"].is_string() && e["params"].is_object(), "{e}");
                e["text"].as_str().expect("Text der Meldung")
            }
        }
    }

    fn is_error_text(e: &Value) -> bool {
        !error_text(e).is_empty()
    }

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
    fn home_is_set_or_empty() {
        let h = home();
        if std::env::var_os("HOME").is_some() {
            assert!(!h.as_os_str().is_empty());
        }
    }

    #[test]
    fn home_prefers_home_over_userprofile() {
        let os = |s: &str| Some(OsString::from(s));
        assert_eq!(home_from(os("/h"), os("C:\\u")), PathBuf::from("/h"));
        assert_eq!(home_from(None, os("C:\\u")), PathBuf::from("C:\\u"));
        assert_eq!(home_from(None, None), PathBuf::new());
    }

    #[test]
    fn series_folder_is_created_or_error_reported() {
        let dir = tmp("lib");
        let lib = ensure_library_dir(&dir.join("neu")).unwrap();
        assert!(lib.is_dir() && lib.ends_with("Zeitreihen"));
        // Already exists: still success.
        assert_eq!(ensure_library_dir(&dir.join("neu")).unwrap(), lib);
        // Data directory is a file: error with path.
        let not_a_dir = dir.join("datei");
        std::fs::write(&not_a_dir, "x").unwrap();
        let e = ensure_library_dir(&not_a_dir).unwrap_err();
        assert!(e.contains("Zeitreihen"), "{e}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn base_dir_takes_folder_of_the_project_file() {
        assert_eq!(base_dir(Some("/a/b/plan.vplan")), PathBuf::from("/a/b"));
        assert_eq!(base_dir(None), home());
        assert_eq!(base_dir(Some("plan.vplan")), PathBuf::from(""));
    }

    #[test]
    fn plan_name_falls_back_to_default() {
        assert_eq!(
            plan_name(&json!({"project": {"name": "Meine Planung"}}), "Planung"),
            "Meine Planung"
        );
        assert_eq!(plan_name(&json!({"project": {"name": "  "}}), "Planung"), "Planung");
        assert_eq!(plan_name(&json!({"project": {"name": 5}}), "Planung"), "Planung");
        assert_eq!(plan_name(&json!({}), "Planung"), "Planung");
        assert_eq!(
            plan_name(&json!({"project": {"name": "Zürich ✓"}}), "Planung"),
            "Zürich ✓"
        );
    }

    #[test]
    fn extension_is_added_only_when_missing() {
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
    fn legacy_base_finds_file_in_folder_or_parent() {
        let root = tmp("legacy");
        let sub = root.join("config");
        std::fs::create_dir_all(&sub).unwrap();
        let cfg = sub.join("config.json");
        let text = r#"{"WealthManagement":{"HistoricalData":"data.csv"}}"#;
        // File nowhere present: folder of the config.json.
        assert_eq!(legacy_base(&cfg, text), sub);
        // File in the parent folder.
        std::fs::write(root.join("data.csv"), "x").unwrap();
        assert_eq!(legacy_base(&cfg, text), root);
        // File in the same folder takes precedence.
        std::fs::write(sub.join("data.csv"), "x").unwrap();
        assert_eq!(legacy_base(&cfg, text), sub);
        // Invalid JSON or missing entry.
        assert_eq!(legacy_base(&cfg, "kein json"), sub);
        assert_eq!(legacy_base(&cfg, "{}"), sub);
        assert_eq!(legacy_base(Path::new("config.json"), "{}"), PathBuf::from(""));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn app_info_returns_version_and_build_time() {
        let (_app, w, _g) = setup();
        let v = call(&w, "app_info", json!({})).unwrap();
        assert!(v["version"].as_str().is_some_and(|s| !s.is_empty()));
        assert!(v["buildEpoch"].as_u64().unwrap() > 0);
    }

    #[test]
    fn example_plan_validates_without_errors() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        assert!(plan.is_object());
        let issues = call(&w, "validate", json!({"plan": plan})).unwrap();
        assert!(issues.is_array());
    }

    #[test]
    fn validate_reports_problems_for_empty_plan() {
        let (_app, w, _g) = setup();
        let issues = call(&w, "validate", json!({"plan": {}})).unwrap();
        assert!(!issues.as_array().unwrap().is_empty());
    }

    #[test]
    fn progress_and_cancel() {
        let (_app, w, _g) = setup();
        assert!(call(&w, "progress_get", json!({})).unwrap().is_object());
        assert!(call(&w, "cancel_job", json!({})).unwrap().is_null());
    }

    #[test]
    fn opt_params_returns_catalog() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let v = call(&w, "opt_params", json!({"plan": plan})).unwrap();
        assert!(v.is_array());
    }

    #[test]
    fn preview_and_simulation_run_for_example_plan() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let p = call(&w, "preview", json!({"plan": plan, "path": null})).unwrap();
        assert!(p.is_object());
        let r = call(&w, "run_simulation", json!({"plan": plan})).unwrap();
        assert!(r.is_object());
    }

    #[test]
    fn rerun_paths_recalculates_single_runs_and_reports_errors() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let r = call(&w, "rerun_paths", json!({"plan": plan, "picks": [[50, 0]]})).unwrap();
        assert!(r["error"].is_null(), "{r}");
        assert_eq!(r["details"].as_array().map(|a| a.len()), Some(1), "{r}");
        assert_eq!(r["details"][0][0], 50);
        let r = call(&w, "rerun_paths", json!({"plan": {}, "picks": []})).unwrap();
        assert!(
            r["error"]["text"].is_string() || r["details"].as_array().is_some_and(|a| a.is_empty()),
            "{r}"
        );
    }

    #[test]
    fn bench_plan_measures_or_reports_error() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let ok = call(&w, "bench_plan", json!({"plan": plan}));
        assert!(ok.is_ok() || ok.is_err());
        assert!(call(&w, "bench_plan", json!({"plan": {}})).is_err());
    }

    #[test]
    fn optimize_returns_report_and_reports_invalid_request() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let r = call(&w, "optimize", json!({"plan": plan, "request": {}})).unwrap();
        assert!(r["outcomes"].is_array());
        let e = call(&w, "optimize", json!({"plan": plan, "request": "unsinn"})).unwrap_err();
        assert!(is_error_text(&e));
    }

    #[test]
    fn series_info_error_for_missing_file() {
        let (_app, w, _g) = setup();
        let e = call(
            &w,
            "series_info",
            json!({"path": "/gibt/es/nicht.csv", "planPath": null}),
        )
        .unwrap_err();
        assert!(is_error_text(&e));
    }

    #[test]
    fn series_info_reads_csv() {
        let (_app, w, _g) = setup();
        let dir = tmp("series");
        let csv = dir.join("s.csv");
        std::fs::write(&csv, "Date,Value\n2000-01,1.0\n2000-02,1.1\n2000-03,1.2\n").unwrap();
        let r = call(
            &w,
            "series_info",
            json!({"path": csv.to_string_lossy(), "planPath": dir.join("p.vplan").to_string_lossy()}),
        );
        // The CSV format depends on the engine parser; both outcomes go through the command.
        assert!(r.is_ok() || is_error_text(&r.unwrap_err()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_are_saved_to_and_read_from_file() {
        let dir = tmp("settings");
        assert!(read_settings(&dir).is_empty());
        write_setting(&dir, "showTaxCols", json!(true)).unwrap();
        write_setting(&dir, "andere", json!("x")).unwrap();
        let all = read_settings(&dir);
        assert_eq!(all["showTaxCols"], json!(true));
        assert_eq!(all["andere"], json!("x"));
        // unreadable file: defaults, and writing overwrites it
        std::fs::write(dir.join(SETTINGS_FILE), "kein json").unwrap();
        assert!(read_settings(&dir).is_empty());
        std::fs::write(dir.join(SETTINGS_FILE), "[1]").unwrap();
        assert!(read_settings(&dir).is_empty());
        write_setting(&dir, "showTaxCols", json!(false)).unwrap();
        assert_eq!(read_settings(&dir)["showTaxCols"], json!(false));
        // data directory not writable (is a file)
        let file = dir.join("datei");
        std::fs::write(&file, "x").unwrap();
        assert!(write_setting(&file, "k", json!(1)).is_err());
    }

    #[test]
    fn settings_via_the_commands() {
        let (_app, w, _g) = setup();
        call(&w, "settings_set", json!({ "key": "showTaxCols", "value": true })).unwrap();
        let v = call(&w, "settings_get", json!({})).unwrap();
        assert_eq!(v["showTaxCols"], json!(true));
    }

    #[test]
    fn library_list_shows_shipped_series() {
        let (_app, w, _g) = setup();
        let v = call(&w, "library_list", json!({})).unwrap();
        assert!(v["dir"].as_str().unwrap().ends_with("Zeitreihen"));
        assert!(v["entries"].is_array());
    }

    #[test]
    fn library_in_own_folder_and_error_for_file() {
        let dir = tmp("list");
        let lib = ensure_library_dir(&dir).unwrap();
        let v = list_in(&lib).unwrap();
        assert_eq!(v.dir, lib.to_string_lossy());
        assert!(!v.entries.is_empty(), "ausgelieferte Zeitreihen werden angelegt");
        // A folder path that is a file cannot be read.
        let not_a_dir = dir.join("datei");
        std::fs::write(&not_a_dir, "x").unwrap();
        assert!(list_in(&not_a_dir).err().unwrap().text().contains("datei"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn library_dir_is_created() {
        let (app, _w, _g) = setup();
        let d = library_dir(app.handle()).unwrap();
        assert!(d.is_dir());
    }

    /// #142: response without time fields (computing time, build time).
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

    /// #142: replace the placeholders of the table with small plans.
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

    /// #142/AK1, AK2, AK4: Every calculation command of the table returns through the Tauri IPC (arguments as the user
    /// interface sends them, camelCase) the same response as the dispatcher of the development server (`engine::dispatch::call`, which
    /// `devserver::call` calls unchanged). Included (AK4): `run_simulation` with `jobId` and then
    /// `progress_get` with the same id.
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
            // The IPC response arrives as JSON text; serde_json does not always read floating point numbers from it bit-exactly
            // (without `float_roundtrip`). The dispatcher's response therefore takes the same path through text.
            let mut dev = engine::dispatch::call(cmd, &args, &lib, &home())
                .map(|v| serde_json::from_str::<Value>(&v.to_string()).unwrap())
                .map_err(|e| e.text().to_string());
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

    /// #142/AK3: The table lists every command from `generate_handler!`, as a calculation command or as «desktop app only»,
    /// and no other. A new command without an entry makes the test fail (counterpart for the
    /// development server: `engine::dispatch`, test `issue_142_ak3_…`).
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

    /// #141: startup option of the smoke test, output and exit code.
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
        // Without --smoke the command does nothing (the app keeps running)
        let (_app, w, _g) = setup();
        assert_eq!(
            call(&w, "smoke_done", json!({"ok": false, "message": "x"})).unwrap(),
            Value::Null
        );
    }

    /// #162/AK1: `smoke_report` also writes the smoke test report to the file from `VIVAPLAN_SMOKE_OUT` (on
    /// Windows the app has no console; `pakete.yml` reads the file).
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

    /// #141/AK3: paths on Windows (drive, backslash, UNC, spaces and umlauts, .rplan → .vplan). The cases
    /// are compiled on all systems, run only on Windows (only there backslash and drive count as a path).
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

    /// #141: spaces and umlauts in paths with «/» (all systems).
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
    fn save_plan_with_path_and_read_again() {
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
    fn saving_plan_into_missing_folder_is_error() {
        let (_app, w, _g) = setup();
        let e = call(&w, "save_plan", json!({"plan": {}, "path": "/gibt/es/nicht/p.vplan"})).unwrap_err();
        assert!(error_text(&e).contains("p.vplan"));
    }

    /// #136/AK4: If saving fails, the command reports the error (the user interface shows it and leaves the
    /// plan unsaved, e2e «#136/AK4»), and the existing file keeps its old content.
    #[cfg(unix)]
    #[test]
    fn issue_136_ak4_failed_save_reports_error_and_keeps_file() {
        use std::os::unix::fs::PermissionsExt;
        let (_app, w, _g) = setup();
        let dir = tmp("save-readonly");
        let file = dir.join("p.vplan");
        let old_plan = json!({"format": "vivaplan/1", "marker": "alt"});
        call(
            &w,
            "save_plan",
            json!({"plan": old_plan, "path": file.to_string_lossy()}),
        )
        .unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();
        let e = call(
            &w,
            "save_plan",
            json!({"plan": {"format": "vivaplan/1", "marker": "neu"}, "path": file.to_string_lossy()}),
        )
        .unwrap_err();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(error_text(&e).contains("p.vplan"), "{e}");
        assert_eq!(api::read_plan(&file).unwrap(), old_plan);
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 1, "keine temporäre Datei");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    #[test]
    fn open_plan_reads_file_and_reports_errors() {
        let dir = tmp("open");
        let f = dir.join("p.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1", "marker": 1}"#).unwrap();
        let o = opened_from(FilePath::Path(f.clone())).unwrap();
        assert_eq!(o.path, f.to_string_lossy());
        assert_eq!(o.plan["marker"], 1);
        // #144/AK2: A file from a newer version is rejected with the format id in the message.
        std::fs::write(&f, r#"{"format": "vivaplan/2"}"#).unwrap();
        assert!(opened_from(FilePath::Path(f.clone()))
            .err()
            .unwrap()
            .text()
            .contains("The file has the format “vivaplan/2”. It requires a newer version of VivaPlan"));
        // Old config.json is rejected.
        let old_config = dir.join("alt.json");
        std::fs::write(&old_config, r#"{"General": {}}"#).unwrap();
        assert!(opened_from(FilePath::Path(old_config))
            .err()
            .unwrap()
            .text()
            .contains("Import old config.json"));
        // No JSON, missing file, path that cannot be converted.
        let broken = dir.join("k.vplan");
        std::fs::write(&broken, "{").unwrap();
        assert!(opened_from(FilePath::Path(broken)).is_err());
        assert!(opened_from(FilePath::Path(dir.join("fehlt"))).is_err());
        assert!(opened_from(FilePath::Url("https://example.com/x".parse().unwrap())).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_to_writes_and_reports_path() {
        let dir = tmp("write");
        let f = dir.join("a.vplan");
        assert_eq!(write_to(&f, &json!({"a": 1})).unwrap(), f.to_string_lossy());
        assert!(write_to(&dir.join("x/y.vplan"), &json!({})).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn series_choice_returns_path_as_text() {
        assert_eq!(
            series_choice(FilePath::Path(PathBuf::from("/a/b.csv"))).unwrap(),
            "/a/b.csv"
        );
        assert!(series_choice(FilePath::Url("https://example.com/x".parse().unwrap())).is_err());
    }

    #[test]
    fn legacy_config_is_imported() {
        let src = root().join("crates/engine/tests/fixtures/legacy_fixed.json");
        let r = legacy_from(FilePath::Path(src), 2026).unwrap();
        assert!(r.plan.is_object());
        assert!(legacy_from(FilePath::Path(PathBuf::from("/gibt/es/nicht.json")), 2026)
            .err()
            .unwrap()
            .text()
            .contains("nicht.json"));
        let dir = tmp("legacy-bad");
        let bad = dir.join("c.json");
        std::fs::write(&bad, "kein json").unwrap();
        assert!(legacy_from(FilePath::Path(bad), 2026).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn shiller_import_from_table_and_error_case() {
        let lib = tmp("shiller");
        // #94: the original table exists only locally; without it this part is skipped.
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
    fn snb_import_error_cases() {
        let lib = tmp("snb");
        assert!(snb_from(&lib, vec![FilePath::Path(lib.join("fehlt.csv"))]).is_err());
        assert!(snb_from(&lib, vec![FilePath::Url("https://example.com/x".parse().unwrap())]).is_err());
        let _ = std::fs::remove_dir_all(&lib);
    }

    /// All own menu entries of this system; «Über» of the application menu (macOS) is the same command as in the
    /// menu «Hilfe» and counts only once.
    fn own_entries() -> Vec<Entry> {
        let mut v = file_entries();
        v.extend(EDIT_MENU.iter().chain(&HELP_MENU).copied());
        #[cfg(target_os = "macos")]
        v.extend(APP_MENU.iter().filter(|e| e.0 != "about").copied());
        v
    }

    /// #170: menu entry with its German text from the text table.
    fn de(e: &Entry) -> (&'static str, String, &'static str) {
        (e.0, i18n::tr("de", e.1), e.2)
    }

    /// #59/AK1–AK4: settings on macOS in the application menu, otherwise at the end of «Datei»; exactly once.
    #[test]
    fn issue_59_settings_per_system() {
        let n = own_entries().iter().filter(|e| e.0 == "settings").count();
        assert_eq!(n, 1, "Einstellungen genau einmal");
        assert_eq!(de(&SETTINGS), ("settings", "Einstellungen…".into(), "CmdOrCtrl+,"));
        let file = file_entries();
        #[cfg(target_os = "macos")]
        {
            assert_eq!(APP_MENU, [("about", "menu.about", ""), SEP, SETTINGS, SEP]);
            assert_eq!(de(&APP_MENU[0]), ("about", "Über VivaPlan".into(), ""));
            assert!(!file.contains(&SETTINGS), "nicht im Menü «Datei»");
            assert_eq!(
                de(file.last().unwrap()),
                ("run", "Simulation starten".into(), "CmdOrCtrl+Enter")
            );
        }
        #[cfg(not(target_os = "macos"))]
        assert_eq!(
            file[file.len() - 2..],
            [SEP, SETTINGS],
            "am Ende von «Datei», danach folgt «Beenden»"
        );
    }

    #[test]
    fn menu_tables_are_consistent() {
        let mut ids = Vec::new();
        for (id, text, key) in &own_entries() {
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
    fn choice_without_match_runs_nothing() {
        // `Err` as follow-up step: had it been called on cancel, the result would be an error.
        let r = picked(|| None::<String>, |e| Err::<u8, CmdError>(e.into()));
        assert_eq!(r, Ok(None));
        assert_eq!(picked(|| Some(2), |x| Ok(x * 2)), Ok(Some(4)));
        assert_eq!(
            picked(|| Some("fehler".to_string()), |e| Err::<u8, CmdError>(e.into())),
            Err(CmdError::Text("fehler".into()))
        );
    }

    /// Result of `save_with` along with what the dialog observed: (result, asked, suggested name).
    type Saved = (Result<Option<String>, CmdError>, bool, String);

    fn save_with_answer(plan: &Value, path: Option<String>, answer: Option<FilePath>) -> Saved {
        let mut was_asked = false;
        let mut suggestion = String::new();
        let r = save_with(plan, path, "Planung", |n| {
            was_asked = true;
            suggestion = n.to_string();
            answer
        });
        (r, was_asked, suggestion)
    }

    #[test]
    fn save_with_dialog_adds_extension_and_respects_cancel() {
        let dir = tmp("save-dialog");
        let plan = json!({"project": {"name": "Mein Plan"}});
        // Cancelled in the dialog: nothing written.
        let (r, was_asked, _) = save_with_answer(&plan, None, None);
        assert_eq!((r, was_asked), (Ok(None), true));
        // Dialog returns a path without extension; the suggested name comes from the project name.
        let (r, _, suggestion) = save_with_answer(&plan, None, Some(FilePath::Path(dir.join("ziel"))));
        assert_eq!(suggestion, "Mein Plan");
        assert_eq!(r.unwrap().unwrap(), dir.join("ziel.vplan").to_string_lossy());
        assert!(dir.join("ziel.vplan").is_file());
        // With a path the dialog is not asked.
        let p = dir.join("direkt.vplan").to_string_lossy().into_owned();
        let (r, was_asked, _) = save_with_answer(&plan, Some(p.clone()), None);
        assert_eq!(r, Ok(Some(p)));
        assert!(!was_asked, "mit Pfad kein Dialog");
        // A choice that cannot be converted is an error.
        let url = FilePath::Url("https://example.com/x".parse().unwrap());
        assert!(save_with_answer(&plan, None, Some(url)).0.is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn commands_reject_missing_arguments() {
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
    fn unknown_command_is_rejected() {
        let (_app, w, _g) = setup();
        assert!(call(&w, "gibt_es_nicht", json!({})).is_err());
    }

    #[test]
    fn simulation_of_example_file_returns_result_without_errors() {
        let (_app, w, _g) = setup();
        let file = root().join("crates/engine/assets/example.vplan");
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
    fn series_in_plan_folder_is_resolved_relatively() {
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
        // Without a plan path (base: home) it is not found.
        assert!(call(&w, "series_info", json!({"path": "renditen-muster.csv"})).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn progress_after_calculation_and_cancel_without_run() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        call(&w, "run_simulation", json!({"plan": plan})).unwrap();
        let p = call(&w, "progress_get", json!({})).unwrap();
        assert_eq!(p["active"], false);
        assert!(p["done"].as_u64().unwrap() >= 1);
        assert!(p["total"].as_u64().unwrap() >= 1);
        // Without a running calculation a cancel has no effect.
        call(&w, "cancel_job", json!({})).unwrap();
        let p = call(&w, "progress_get", json!({})).unwrap();
        assert_eq!(p["cancelled"], false);
    }

    #[test]
    fn progress_and_cancel_follow_the_job_id() {
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
    fn issue_106_cancel_job_cancels_the_calculation_with_this_id() {
        let (_app, w, _g) = setup();
        let job = engine::progress::start_as(Some(55_001), 0);
        let other = engine::progress::start_as(Some(55_002), 0);
        assert!(!engine::progress::cancelled());
        // The inner job (55002) is the one of the thread; cancelling 55001 does not affect it.
        call(&w, "cancel_job", json!({"jobId": 55_001})).unwrap();
        assert!(!engine::progress::cancelled());
        assert!(engine::progress::snapshot_of(55_001).cancelled);
        // Without an id all running calculations are cancelled.
        call(&w, "cancel_job", json!({})).unwrap();
        assert!(engine::progress::cancelled());
        drop(other);
        drop(job);
    }

    #[test]
    fn issue_106_setting_is_saved_and_read_via_the_command() {
        let (_app, w, _g) = setup();
        // Clean up first: an earlier cancelled run (e.g. by a mutant) may have left the value behind.
        call(&w, "settings_set", json!({"key": "issue_106Test", "value": null})).unwrap();
        let before = call(&w, "settings_get", json!({})).unwrap();
        assert!(before.get("issue_106Test").is_none_or(|v| v.is_null()), "{before}");
        call(&w, "settings_set", json!({"key": "issue_106Test", "value": 42})).unwrap();
        let after = call(&w, "settings_get", json!({})).unwrap();
        assert_eq!(after["issue_106Test"], 42);
        call(&w, "settings_set", json!({"key": "issue_106Test", "value": null})).unwrap();
    }

    /// #163/AK1: The data folder belongs to the test process (process id in the name) and is removed after the test.
    #[test]
    fn issue_163_ak1_data_folder_per_test_process_and_removed_after_the_test() {
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
    fn issue_106_data_folders_name_the_identifier() {
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
    fn issue_106_file_reported_by_the_system_is_kept_pending() {
        // macOS: double-click on a plan file reports «Opened» (#44); other files are ignored.
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
    fn example_plan_has_no_validation_errors_and_strategies() {
        let (_app, w, _g) = setup();
        let plan = call(&w, "example_plan", json!({})).unwrap();
        let issues = call(&w, "validate", json!({"plan": plan})).unwrap();
        assert!(
            issues.as_array().unwrap().iter().all(|i| i["severity"] != "error"),
            "{issues}"
        );
        let catalog = call(&w, "opt_params", json!({"plan": plan})).unwrap();
        assert!(catalog.as_array().unwrap().len() >= 2);
    }

    #[test]
    fn invalid_plan_is_reported_in_result() {
        let (_app, w, _g) = setup();
        let r = call(&w, "run_simulation", json!({"plan": {}})).unwrap();
        assert!(!r["error"].is_null() || !r["issues"].as_array().unwrap().is_empty());
        let p = call(&w, "preview", json!({"plan": {}})).unwrap();
        assert!(!p["error"].is_null() || !p["issues"].as_array().unwrap().is_empty());
        assert!(p["result"].is_null());
    }

    #[test]
    fn menu_shortcuts_have_valid_form() {
        const MODIFIERS: [&str; 3] = ["CmdOrCtrl", "Shift", "Alt"];
        for (id, _, key) in &own_entries() {
            if key.is_empty() {
                continue;
            }
            let parts: Vec<&str> = key.split('+').collect();
            let (last_key, mods) = parts.split_last().unwrap();
            assert!(!last_key.is_empty(), "{id}: Taste fehlt");
            assert!(mods.iter().all(|m| MODIFIERS.contains(m)), "{id}: {key}");
        }
        // No shortcut assigned twice.
        let mut keys: Vec<&str> = own_entries().iter().map(|e| e.2).filter(|k| !k.is_empty()).collect();
        let n = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), n, "doppeltes Tastenkürzel");
    }

    #[test]
    fn menu_titles_and_application_menu() {
        let titles = [FILE_TITLE, EDIT_TITLE, VIEW_TITLE, HELP_TITLE].map(|k| i18n::tr("de", k));
        assert_eq!(titles, ["Datei", "Bearbeiten", "Ansicht", "Hilfe"]);
        #[cfg(target_os = "macos")]
        {
            assert_eq!(APP_MENU[0].0, "about");
            assert_eq!(APP_MENU[1], SEP);
            assert_eq!(APP_MENU[3], SEP);
        }
    }

    #[test]
    fn menu_event_is_reported_to_the_ui_with_id() {
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
    fn real_dialogs_are_registered_as_service() {
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
    fn dialog_descriptions_are_unchanged() {
        let cases = [
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
        for (d, title, filter, ext) in cases {
            // #170: the German texts come from the text table
            assert_eq!(
                (
                    i18n::tr("de", d.title),
                    i18n::tr("de", d.filter),
                    d.extensions.join(",")
                ),
                (title.to_string(), filter.to_string(), ext.to_string())
            );
        }
    }

    fn chosen(p: &Path) -> FakeDialogs {
        FakeDialogs {
            file: Some(FilePath::Path(p.to_path_buf())),
            ..Default::default()
        }
    }

    #[test]
    fn cancel_in_dialog_returns_null_for_all_dialog_commands() {
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
    fn open_plan_via_command() {
        let dir = tmp("cmd-open");
        let f = dir.join("p.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1", "marker": 1}"#).unwrap();
        let (_app, w, _g, _) = setup_with(chosen(&f));
        let r = call(&w, "open_plan", json!({})).unwrap();
        assert_eq!(r["path"], f.to_string_lossy().as_ref());
        assert_eq!(r["plan"]["marker"], 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn open_plan_via_command_reports_errors() {
        let dir = tmp("cmd-open-err");
        let (_app, w, _g, _) = setup_with(chosen(&dir.join("fehlt.vplan")));
        assert!(is_error_text(&call(&w, "open_plan", json!({})).unwrap_err()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_via_command_with_dialog_and_suggested_name() {
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
        // With a path nothing is asked.
        let direct_path = dir.join("direkt.vplan");
        call(
            &w,
            "save_plan",
            json!({"plan": plan, "path": direct_path.to_string_lossy()}),
        )
        .unwrap();
        assert_eq!(fake.asked().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #44/AK3 (legacy: RetirementPlanner): An opened .rplan file is never overwritten. «Speichern» asks like «Speichern unter» and
    /// suggests the same name with .vplan; the .vplan can be opened again, the .rplan stays unchanged.
    #[test]
    fn rplan_save_suggests_vplan_and_leaves_old_file_unchanged() {
        let dir = tmp("cmd-save-legacy");
        let legacy_file = dir.join("Demo.rplan");
        let content = r#"{"format": "pensionsplaner/1", "project": {"name": "X"}}"#;
        std::fs::write(&legacy_file, content).unwrap();
        let fake = FakeDialogs {
            save: Some(FilePath::Path(dir.join("Demo.rplan"))),
            ..Default::default()
        };
        let (_app, w, _g, fake) = setup_with(fake);
        let plan = json!({"format": "vivaplan/1", "project": {"name": "X"}});
        let r = call(
            &w,
            "save_plan",
            json!({"plan": plan, "path": legacy_file.to_string_lossy()}),
        )
        .unwrap();
        // Even if .rplan is chosen again in the dialog, a .vplan file is created.
        assert_eq!(r.as_str().unwrap(), dir.join("Demo.vplan").to_string_lossy());
        assert_eq!(
            fake.asked(),
            vec!["save|Planung speichern|VivaPlan-Plan|vplan|Demo.vplan"]
        );
        assert_eq!(std::fs::read_to_string(&legacy_file).unwrap(), content);
        let reopened = call(&w, "open_path", json!({"path": r.as_str().unwrap()})).unwrap();
        assert_eq!(reopened["plan"], plan);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// legacy: RetirementPlanner – cancelled saving of a .rplan file.
    #[test]
    fn cancelled_rplan_save_writes_nothing() {
        let dir = tmp("cmd-save-legacy-cancel");
        let legacy_file = dir.join("a.RPLAN");
        std::fs::write(&legacy_file, "{}").unwrap();
        let (_app, w, _g, fake) = setup_with(FakeDialogs::default());
        let r = call(
            &w,
            "save_plan",
            json!({"plan": {"x": 1}, "path": legacy_file.to_string_lossy()}),
        )
        .unwrap();
        assert!(r.is_null());
        assert_eq!(fake.asked(), vec!["save|Planung speichern|VivaPlan-Plan|vplan|a.vplan"]);
        assert_eq!(std::fs::read_to_string(&legacy_file).unwrap(), "{}");
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

    /// #44/AK2: window title «VivaPlan - ‹file name›», without a file just «VivaPlan».
    #[test]
    fn window_title_with_and_without_file() {
        assert_eq!(window_title(None), "VivaPlan");
        assert_eq!(window_title(Some("")), "VivaPlan");
        assert_eq!(window_title(Some("/a/b/Mein Plan.vplan")), "VivaPlan - Mein Plan.vplan");
        let (_app, w, _g) = setup();
        let t = call(&w, "set_window_title", json!({"path": "/a/Demo.rplan"})).unwrap();
        assert_eq!(t, "VivaPlan - Demo.rplan");
        assert_eq!(call(&w, "set_window_title", json!({})).unwrap(), "VivaPlan");
    }

    /// #44/AK5: A plan passed at startup (double-click, file association) is opened and remembered.
    #[test]
    fn file_passed_at_startup_is_opened() {
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
        // While the user interface runs, «open_path» opens the file and clears the pending file.
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
    fn legacy_config_via_command() {
        let src = root().join("crates/engine/tests/fixtures/legacy_fixed.json");
        let (_app, w, _g, fake) = setup_with(chosen(&src));
        let r = call(&w, "import_legacy", json!({"startYear": 2026})).unwrap();
        assert!(r["plan"].is_object());
        assert_eq!(fake.asked().len(), 1);
        // Year missing: argument error before the dialog.
        assert!(call(&w, "import_legacy", json!({})).is_err());
        assert_eq!(fake.asked().len(), 1);
    }

    #[test]
    fn legacy_config_via_command_reports_errors() {
        let dir = tmp("cmd-legacy-err");
        let bad = dir.join("c.json");
        std::fs::write(&bad, "kein json").unwrap();
        let (_app, w, _g, _) = setup_with(chosen(&bad));
        assert!(is_error_text(
            &call(&w, "import_legacy", json!({"startYear": 2026})).unwrap_err()
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn choose_series_via_command() {
        {
            let (_app, w, _g, _) = setup_with(chosen(Path::new("/a/b.csv")));
            assert_eq!(call(&w, "pick_series", json!({})).unwrap(), "/a/b.csv");
        }
        let fake = FakeDialogs {
            file: Some(FilePath::Url("https://example.com/x".parse().unwrap())),
            ..Default::default()
        };
        let (_app, w, _g, _) = setup_with(fake);
        assert!(is_error_text(&call(&w, "pick_series", json!({})).unwrap_err()));
    }

    #[test]
    fn shiller_import_via_command_reports_file_errors() {
        let dir = tmp("cmd-shiller");
        let (_app, w, _g, fake) = setup_with(chosen(&dir.join("fehlt.xls")));
        assert!(is_error_text(&call(&w, "import_shiller", json!({})).unwrap_err()));
        assert_eq!(fake.asked().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn snb_import_via_command_reports_file_errors() {
        let dir = tmp("cmd-snb");
        let fake = FakeDialogs {
            files: Some(vec![FilePath::Path(dir.join("fehlt.csv"))]),
            ..Default::default()
        };
        let (_app, w, _g, fake) = setup_with(fake);
        assert!(is_error_text(&call(&w, "import_snb", json!({})).unwrap_err()));
        assert_eq!(fake.asked().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn snb_import_from_the_shipped_cubes() {
        let lib = tmp("snb-ok");
        let cubes = ["devkum", "plkopr", "zimoma", "rendoblim"]
            .map(|n| FilePath::Path(root().join(format!("crates/engine/data/quellen/snb-{n}.csv"))));
        let rep = snb_from(&lib, cubes.to_vec()).unwrap();
        assert!(!rep.written.is_empty());
        let _ = std::fs::remove_dir_all(&lib);
    }

    fn example_stub() -> Value {
        json!({"format": "vivaplan/1", "marker": "beispiel"})
    }

    #[test]
    fn start_without_remembered_path_opens_example_without_notice() {
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

    /// #70/AK1, AK6: first start only without a settings file; afterwards (even if the remembered file is missing) no longer.
    #[test]
    fn issue_70_first_start_only_without_settings() {
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
        // With a passed file (double-click) there is no welcome dialog.
        let dir = tmp("fea54-datei");
        let f = dir.join("x.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1", "marker": "x"}"#).unwrap();
        assert!(!startup_from(&dir, Some(f), example_stub).first);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// #70/AK7, #71/AK6: menu «Hilfe» has «Willkommen …» and «Erste Schritte».
    #[test]
    fn issue_70_issue_71_help_menu() {
        let help: Vec<_> = HELP_MENU.iter().map(de).collect();
        assert!(help.contains(&("welcome", "Willkommen …".into(), "")));
        assert!(help.contains(&("steps", "Erste Schritte".into(), "")));
    }

    #[test]
    fn start_opens_remembered_file() {
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
    fn start_with_deleted_file_shows_example_and_notice() {
        let dir = tmp("start-fehlt");
        let f = dir.join("weg.vplan");
        write_setting(&dir, LAST_PLAN_KEY, json!(f.to_string_lossy())).unwrap();
        let s = startup_from(&dir, None, example_stub);
        assert!(s.path.is_none());
        assert_eq!(s.plan["marker"], "beispiel");
        let n = s.notice.unwrap();
        // #170: the notice is a key of the text table with the file name; the user interface translates it
        assert_eq!(
            n,
            Notice {
                key: "startup.notFound",
                file: "weg.vplan".into()
            }
        );
        assert!(i18n::tr("de", n.key).contains("nicht gefunden"));
        // The remembered path is kept
        assert_eq!(read_settings(&dir)[LAST_PLAN_KEY], json!(f.to_string_lossy()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn start_with_unreadable_file_shows_example_and_keeps_remembered_path() {
        let dir = tmp("start-broken");
        let f = dir.join("broken.vplan");
        std::fs::write(&f, "kein json").unwrap();
        write_setting(&dir, LAST_PLAN_KEY, json!(f.to_string_lossy())).unwrap();
        let s = startup_from(&dir, None, example_stub);
        assert!(s.path.is_none());
        assert_eq!(s.notice.unwrap().key, "startup.unreadable");
        assert_eq!(read_settings(&dir)[LAST_PLAN_KEY], json!(f.to_string_lossy()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remembered_path_is_opened_at_startup_and_replaced() {
        let dir = tmp("merken");
        let (a, b) = (dir.join("a.vplan"), dir.join("b.vplan"));
        std::fs::write(&a, r#"{"format": "vivaplan/1", "marker": "a"}"#).unwrap();
        std::fs::write(&b, r#"{"format": "vivaplan/1", "marker": "b"}"#).unwrap();
        remember_in(&dir, &a.to_string_lossy());
        assert_eq!(startup_from(&dir, None, example_stub).plan["marker"], "a");
        remember_in(&dir, &b.to_string_lossy());
        assert_eq!(startup_from(&dir, None, example_stub).plan["marker"], "b");
        // Only the path is remembered
        assert_eq!(read_settings(&dir).len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn open_and_save_via_commands_remember_the_path() {
        let dir = tmp("merken-befehle");
        let f = dir.join("p.vplan");
        std::fs::write(&f, r#"{"format": "vivaplan/1"}"#).unwrap();
        let (_app, w, _g, _) = setup_with(chosen(&f));
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
    fn quit_asks_only_with_unsaved_changes() {
        let (app, w, _g) = setup();
        let q = app.state::<Quit>();
        assert!(!q.must_ask());
        call(&w, "set_dirty", json!({"dirty": true})).unwrap();
        assert!(q.must_ask());
        call(&w, "set_dirty", json!({"dirty": false})).unwrap();
        assert!(!q.must_ask());
    }

    #[test]
    fn confirmed_quit_permits_exit() {
        let q = Quit::default();
        q.dirty.store(true, Ordering::SeqCst);
        q.confirmed.store(true, Ordering::SeqCst);
        assert!(!q.must_ask());
    }

    #[test]
    fn bundled_example_plan_is_readable_or_fallback_applies() {
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
    fn quit_asks_with_changes_and_not_otherwise() {
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
    fn quit_menu_item_asks_with_changes_instead_of_quitting() {
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

    /// #170/AK9: «Ansicht → Sprache» lists «wie System» and every available language in its own language; the
    /// choice is checked, and only the choice.
    #[test]
    fn issue_170_ak9_language_menu_entries() {
        let langs = i18n::languages();
        for choice in std::iter::once(i18n::SYSTEM.to_string()).chain(langs.iter().map(|(c, _)| c.clone())) {
            let e = language_entries("de", &choice);
            assert_eq!(e.len(), langs.len() + 1);
            assert_eq!(e[0].0, "language-system");
            assert_eq!(e[0].1, "wie System");
            for ((id, text, _), (code, name)) in e[1..].iter().zip(&langs) {
                assert_eq!(id, &format!("language-{code}"));
                assert_eq!(text, name, "jede Sprache in ihrer eigenen Sprache");
            }
            let checked: Vec<&str> = e.iter().filter(|x| x.2).map(|x| x.0.as_str()).collect();
            assert_eq!(checked, [format!("language-{choice}")]);
        }
        assert!(langs.iter().any(|(c, n)| c == "de" && n == "Deutsch"));
    }

    /// #172/AK1, AK3: «English» is offered in «Ansicht → Sprache»; menus and file dialogs have English texts.
    #[test]
    fn issue_172_ak3_menus_and_dialogs_in_english() {
        assert!(i18n::languages().contains(&("en".to_string(), "English".to_string())));
        let e = language_entries("en", "en");
        assert_eq!(e[0].1, "as system");
        assert!(e.contains(&("language-en".to_string(), "English".to_string(), true)));
        let tr = |key: &str| i18n::tr("en", key);
        let titles = [
            FILE_TITLE,
            EDIT_TITLE,
            VIEW_TITLE,
            HELP_TITLE,
            QUIT_TEXT,
            "menu.view.language",
        ];
        assert_eq!(titles.map(tr), ["File", "Edit", "View", "Help", "Quit", "Language"]);
        let entries: Vec<Entry> = file_entries().into_iter().chain(EDIT_MENU).chain(HELP_MENU).collect();
        for (id, key, _) in entries.into_iter().filter(|(id, ..)| !id.is_empty()) {
            assert_ne!(tr(key), i18n::tr("de", key), "{id}: {key}");
        }
        assert_eq!(tr("menu.file.saveAs"), "Save as…");
        assert_eq!(tr("menu.help.shortcuts"), "Keyboard shortcuts");
        let dialogs = [
            OPEN_DIALOG,
            SAVE_DIALOG,
            LEGACY_DIALOG,
            SERIES_DIALOG,
            SHILLER_DIALOG,
            SNB_DIALOG,
        ];
        for d in dialogs {
            assert_ne!(tr(d.title), i18n::tr("de", d.title), "{}", d.title);
        }
        assert_eq!(
            (tr(OPEN_DIALOG.title), tr(OPEN_DIALOG.filter)),
            ("Open plan".into(), "VivaPlan plan".into())
        );
    }

    /// #174/AK1, AK3: «Français» is offered in «Ansicht → Sprache»; menus and file dialogs have French texts.
    #[test]
    fn issue_174_ak3_menus_and_dialogs_in_french() {
        assert!(i18n::languages().contains(&("fr".to_string(), "Français".to_string())));
        let e = language_entries("fr", "fr");
        assert_eq!(e[0].1, "comme le système");
        assert!(e.contains(&("language-fr".to_string(), "Français".to_string(), true)));
        let tr = |key: &str| i18n::tr("fr", key);
        let titles = [
            FILE_TITLE,
            EDIT_TITLE,
            VIEW_TITLE,
            HELP_TITLE,
            QUIT_TEXT,
            "menu.view.language",
        ];
        assert_eq!(
            titles.map(tr),
            ["Fichier", "Édition", "Affichage", "Aide", "Quitter", "Langue"]
        );
        let entries: Vec<Entry> = file_entries().into_iter().chain(EDIT_MENU).chain(HELP_MENU).collect();
        for (id, key, _) in entries.into_iter().filter(|(id, ..)| !id.is_empty()) {
            assert_ne!(tr(key), i18n::tr("de", key), "{id}: {key}");
            assert_ne!(tr(key), i18n::tr("en", key), "{id}: {key}");
        }
        assert_eq!(tr("menu.file.saveAs"), "Enregistrer sous…");
        assert_eq!(tr("menu.help.shortcuts"), "Raccourcis clavier");
        let dialogs = [
            OPEN_DIALOG,
            SAVE_DIALOG,
            LEGACY_DIALOG,
            SERIES_DIALOG,
            SHILLER_DIALOG,
            SNB_DIALOG,
        ];
        for d in dialogs {
            assert_ne!(tr(d.title), i18n::tr("de", d.title), "{}", d.title);
        }
        assert_eq!(
            (tr(OPEN_DIALOG.title), tr(OPEN_DIALOG.filter)),
            ("Ouvrir une planification".into(), "Plan VivaPlan".into())
        );
    }

    /// #175/AK1, AK3: «Italiano» is offered in «Ansicht → Sprache»; menus and file dialogs have Italian texts.
    #[test]
    fn issue_175_ak3_menus_and_dialogs_in_italian() {
        assert!(i18n::languages().contains(&("it".to_string(), "Italiano".to_string())));
        let e = language_entries("it", "it");
        assert_eq!(e[0].1, "come il sistema");
        assert!(e.contains(&("language-it".to_string(), "Italiano".to_string(), true)));
        let tr = |key: &str| i18n::tr("it", key);
        let titles = [
            FILE_TITLE,
            EDIT_TITLE,
            VIEW_TITLE,
            HELP_TITLE,
            QUIT_TEXT,
            "menu.view.language",
        ];
        assert_eq!(
            titles.map(tr),
            ["File", "Modifica", "Vista", "Aiuto", "Uscire", "Lingua"]
        );
        let entries: Vec<Entry> = file_entries().into_iter().chain(EDIT_MENU).chain(HELP_MENU).collect();
        for (id, key, _) in entries.into_iter().filter(|(id, ..)| !id.is_empty()) {
            assert_ne!(tr(key), i18n::tr("de", key), "{id}: {key}");
            assert_ne!(tr(key), i18n::tr("en", key), "{id}: {key}");
            assert_ne!(tr(key), i18n::tr("fr", key), "{id}: {key}");
        }
        assert_eq!(tr("menu.file.saveAs"), "Salvare con nome…");
        assert_eq!(tr("menu.help.shortcuts"), "Scelte rapide da tastiera");
        let dialogs = [
            OPEN_DIALOG,
            SAVE_DIALOG,
            LEGACY_DIALOG,
            SERIES_DIALOG,
            SHILLER_DIALOG,
            SNB_DIALOG,
        ];
        for d in dialogs {
            assert_ne!(tr(d.title), i18n::tr("de", d.title), "{}", d.title);
        }
        assert_eq!(
            (tr(OPEN_DIALOG.title), tr(OPEN_DIALOG.filter)),
            ("Aprire la pianificazione".into(), "Piano VivaPlan".into())
        );
    }

    /// #170/AK3, AK9: the user interface reports language and choice; menus (rebuilt in the real app, which needs the
    /// main thread of the system and is therefore not testable here) and file dialogs follow.
    #[test]
    fn issue_170_ak9_ui_reports_language_and_choice() {
        let (app, w, _g) = setup();
        // the user interface reports the choice: language state for menus and dialogs
        call(&w, "set_menu_language", json!({ "language": "de-CH", "choice": "de" })).unwrap();
        let ui = app.state::<i18n::UiLanguage>();
        assert_eq!((ui.language(), ui.choice()), ("de".to_string(), "de".to_string()));
        call(
            &w,
            "set_menu_language",
            json!({ "language": "ja", "choice": i18n::SYSTEM }),
        )
        .unwrap();
        assert_eq!(
            (ui.language(), ui.choice()),
            ("de".to_string(), i18n::SYSTEM.to_string())
        );
    }

    /// #170/AK2: only at the very first start (no settings file, #70) the app reports the system language; the menus
    /// start in the saved choice, otherwise in the system language.
    #[test]
    fn issue_170_ak2_first_start_language_and_menu_language_at_startup() {
        let dir = tmp("first-language");
        assert_eq!(first_start_language_in(&dir, || "fr".into()), Some("fr".to_string()));
        let l = startup_language(&dir);
        assert_eq!(l.choice(), i18n::SYSTEM);
        write_setting(&dir, i18n::SETTING, json!("de")).unwrap();
        assert_eq!(first_start_language_in(&dir, || "fr".into()), None);
        let l = startup_language(&dir);
        assert_eq!((l.language(), l.choice()), ("de".to_string(), "de".to_string()));
        write_setting(&dir, i18n::SETTING, json!(i18n::SYSTEM)).unwrap();
        assert_eq!(startup_language(&dir).choice(), i18n::SYSTEM);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn quit_has_its_own_id_and_shortcut() {
        for (id, _, _) in &own_entries() {
            assert_ne!(*id, QUIT_ID);
        }
        assert_eq!(i18n::tr("de", QUIT_TEXT), "Beenden");
    }

    // #124/AK8: The windows from the configuration (there `create: false`) are only created at the start of the app.
    #[test]
    fn windows_from_the_config_are_created() {
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
