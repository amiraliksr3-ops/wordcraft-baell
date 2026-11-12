//! The Android host: `android_main`, the eframe app wrapper and the platform services.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Arc;

use eframe::egui;
use wordcraft_engine::Session;
use wordcraft_ui_egui::{FileDialogAnswer, FileDialogRequest, Services, UiState, WordApp};
use winit::platform::android::EventLoopBuilderExtAndroid as _;

/// Where exports the app asks to download (PDF, HTML and friends) go when there is no system
/// file dialog to name a destination.
const DOWNLOADS: &str = "Downloads";

/// The eframe wrapper, the desktop app's `App` minus what a phone doesn't have: no window
/// geometry (the system owns the window), no native printing, no control server, no updates.
/// The second field is the internal data directory, which [`save_prefs`] needs on exit.
struct BaellApp(WordApp, Option<PathBuf>);

impl eframe::App for BaellApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }

    fn on_exit(&mut self) {
        self.0.session.finish_recovery();
        save_prefs(&self.0, self.1.as_deref());
    }

    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.0.raw_input_hook(ctx, raw);
    }
}

/// The NativeActivity entry point: the glue (`android-activity`, through winit) declares
/// `android_main` with the Rust ABI, so this is a plain function, and `#[unsafe(no_mangle)]`
/// is what lets the shared library export it under that name (see `lib.rs`).
#[unsafe(no_mangle)]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    run(app)
}

/// Runs the app for this activity, from `android_main` (see `lib.rs`).
pub fn run(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("WordCraft")
            .with_max_level(log::LevelFilter::Info),
    );
    log::info!("WordCraft (Baell) {} starting", env!("CARGO_PKG_VERSION"));

    // The activity's internal data directory holds everything the user keeps (preferences,
    // recovery, fonts, the custom dictionary). Without it the app still runs; it just keeps
    // nothing between sessions.
    let data_dir = app.internal_data_path().or_else(|| app.external_data_path());
    match data_dir.as_deref() {
        Some(dir) => log::info!("data directory: {}", dir.display()),
        None => log::warn!("no data directory from the activity; preferences and recovery are off"),
    }

    let event_loop = match winit::event_loop::EventLoop::<eframe::UserEvent>::with_user_event().with_android_app(app).build() {
        Ok(loop_) => loop_,
        Err(e) => {
            log::error!("event loop: {e}");
            return;
        }
    };
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_title("WordCraft"),
        ..Default::default()
    };
    let mut winit_app = eframe::create_native(
        "WordCraft",
        options,
        Box::new(move |cc| creator(cc, data_dir.as_deref())),
        &event_loop,
    );
    if let Err(e) = event_loop.run_app(&mut winit_app) {
        log::error!("event loop: {e}");
    }
}

/// The eframe app creator: builds the session and the front end, the desktop app's startup
/// minus the desktop-only steps (`apps/wordcraft/src/main.rs`, `run_native`'s closure).
fn creator(cc: &eframe::CreationContext<'_>, data_dir: Option<&Path>) -> Result<Box<dyn eframe::App>, Box<dyn core::error::Error + Send + Sync>> {
    let app = build(cc, data_dir).map_err(Box::<dyn core::error::Error + Send + Sync>::from)?;
    Ok(Box::new(BaellApp(app, data_dir.map(Path::to_path_buf))))
}

/// The system region picks the new document's paper, the units and the decimal separator (#838),
/// the same way the desktop and web apps do. Then preferences, the user dictionary, font packs,
/// recovery, in the desktop app's order.
fn build(cc: &eframe::CreationContext<'_>, data_dir: Option<&Path>) -> Result<WordApp, String> {
    let region = wordcraft_ui_egui::i18n::system_region();
    let mut session = Session::new(wordcraft_engine::blank_document(&region));
    session.system_region = region;
    let mut app = WordApp::new(session, services(data_dir));
    load_prefs(&mut app, data_dir);
    load_user_dictionary(&mut app, data_dir);
    // Language packs from the suite's font store; an Install click is the person's own.
    wordcraft_engine::packs::use_font_store(true);
    // A region chosen in File ▸ Options replaces the system's for the first blank document too.
    if app.session.prefs.region.is_some() {
        let doc = app.session.blank_document();
        app.replace_startup_document(doc);
    }
    load_user_fonts(data_dir);
    if let Some(dir) = data_dir {
        let ctx = cc.egui_ctx.clone();
        let repaint = Arc::new(move || ctx.request_repaint());
        if let Err(e) = app.session.enable_recovery(dir.join("recovery"), repaint) {
            log::warn!("local recovery unavailable: {e}");
        }
    }
    Ok(app)
}

/// File ▸ Open and Save As answer at once that the system has no dialog, so the same frame
/// opens the app's own built-in file browser (#576), which works without any native dialog.
fn builtin_file_dialog(req: FileDialogRequest) -> mpsc::Receiver<FileDialogAnswer> {
    let (tx, rx) = mpsc::channel::<FileDialogAnswer>();
    let _ = tx.send(Err(format!("no native file dialog on Android (asked: {req:?}); the built-in file browser opens instead")));
    rx
}

/// Writes a download next to the app's data (see [`DOWNLOADS`]); the file name is the last path
/// segment only, so a hostile name cannot escape the folder.
fn download_service(dir: PathBuf) -> impl Fn(&str, &[u8]) -> Result<(), String> {
    move |name, bytes| {
        let Some(name) = last_segment(name) else { return Err("download: no file name".into()) };
        let out = dir.join(name);
        std::fs::write(&out, bytes).map_err(|e| format!("cannot write {}: {e}", out.display()))
    }
}

/// The file name of a path, or `None` when the path ends in a separator or is empty.
fn last_segment(path: &str) -> Option<&str> {
    let trimmed = path.trim_end_matches(['/', '\\']);
    let segment = trimmed.rsplit(['/', '\\']).next().unwrap_or(trimmed);
    (!segment.is_empty()).then_some(segment)
}

/// The host services a phone can offer: the built-in file browser for dialogs, downloads into
/// the app's own folder, personal templates and pictures the user inserted before.
fn services(data_dir: Option<&Path>) -> Services {
    Services {
        file_dialog: Some(Box::new(builtin_file_dialog)),
        download: data_dir.map(|dir| Box::new(download_service(dir.join(DOWNLOADS))) as Box<dyn Fn(&str, &[u8]) -> Result<(), String>>),
        templates_dir: data_dir.map(|dir| dir.join("Templates")),
        ..Services::default()
    }
}

/// `ui.json` in the data directory: the same preferences file the desktop keeps in its settings
/// folder, so Options choices survive the app coming and going.
fn prefs_path(data_dir: Option<&Path>) -> Option<PathBuf> {
    data_dir.map(|dir| dir.join("ui.json"))
}

fn load_prefs(app: &mut WordApp, data_dir: Option<&Path>) {
    let Some(p) = prefs_path(data_dir) else { return };
    if let Ok(bytes) = std::fs::read(&p)
        && let Some(ui) = UiState::from_json(&bytes)
    {
        app.apply_prefs(ui);
    }
}

fn save_prefs(app: &WordApp, data_dir: Option<&Path>) {
    let Some(p) = prefs_path(data_dir) else { return };
    let bytes = match serde_json::to_vec_pretty(&app.prefs()) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::warn!("preferences: cannot serialize: {e}");
            return;
        }
    };
    if let Err(e) = write_prefs(&p, &bytes) {
        log::warn!("preferences: {e}");
    }
}

/// Writes `bytes` to a sibling temporary file and renames it over `path`, so a failed or
/// interrupted write leaves the previous preferences in place (the desktop app's `write_prefs`).
fn write_prefs(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let name = path.file_name().map_or_else(|| "ui.json".to_string(), |n| n.to_string_lossy().into_owned());
    let tmp = path.with_file_name(format!(".{name}.{}.tmp", std::process::id()));
    let written = std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))
        .and_then(|()| std::fs::write(&tmp, bytes))
        .and_then(|()| std::fs::rename(&tmp, path));
    if written.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    written
}

/// `custom-dictionary.txt` in the data directory: the words Add to Dictionary kept (#40).
fn load_user_dictionary(app: &mut WordApp, data_dir: Option<&Path>) {
    let Some(p) = data_dir.map(|dir| dir.join(Session::USER_DICT_FILE)) else { return };
    wordcraft_proof::load_user_dictionary(&p);
    app.session.user_dict_path = Some(p);
}

/// Fonts the user added (#582) live in `fonts` in the data directory: load them, and store
/// fonts added later there.
fn load_user_fonts(data_dir: Option<&Path>) {
    let Some(dir) = data_dir.map(|dir| dir.join("fonts")) else { return };
    let (loaded, skipped) = wordcraft_fonts::FontDb::global().load_user_fonts(&dir);
    if !loaded.is_empty() || !skipped.is_empty() {
        log::info!("user fonts: {} loaded, {} skipped from {}", loaded.len(), skipped.len(), dir.display());
    }
}
