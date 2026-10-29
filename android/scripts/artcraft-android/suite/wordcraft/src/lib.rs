use eframe::egui;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
use wordcraft_engine::{Region, Session};
use wordcraft_ui_egui::{Services, WordApp};

struct App {
    editor: WordApp,
    prefs: PathBuf,
    saved: String,
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
        self.editor.logic(ctx);
        if let Ok(text) = serde_json::to_string(&self.editor.prefs()) {
            if text != self.saved && std::fs::write(&self.prefs, &text).is_ok() {
                self.saved = text;
            }
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.editor.ui(ui);
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if let Ok(texts) = artcraft_android_platform::committed_text() {
            raw.events.extend(texts.into_iter().map(egui::Event::Text));
        }
        self.editor.raw_input_hook(ctx, raw);
    }
}
#[unsafe(no_mangle)]
fn android_main(android: AndroidApp) {
    // SAFETY: AndroidApp supplies live JVM/activity handles for this process.
    if let Err(error) = unsafe { artcraft_android_platform::initialize(android.vm_as_ptr().cast(), android.activity_as_ptr().cast()) } {
        log::error!("{error}");
        return;
    }

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let Some(private) = android.internal_data_path() else {
        return;
    };
    let documents = PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.wordcraft.ru");
    let home = if std::fs::create_dir_all(&documents).is_ok() {
        documents
    } else {
        private.clone()
    };
    // The built-in file browser starts in the current folder when no desktop home is set.
    if let Err(e) = std::env::set_current_dir(&home) {
        log::warn!("{e}");
    }
    let prefs = private.join("preferences.json");
    let saved = std::fs::read_to_string(&prefs).unwrap_or_default();
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "WordCraft",
        options,
        Box::new(move |_cc| {
            let session = Session::new(wordcraft_engine::blank_document(&Region::default()));
            let services = Services {
                file_dialog: Some(Box::new(|request| {
                    use wordcraft_ui_egui::FileDialogRequest;
                    match request {
                        FileDialogRequest::Open { .. } => artcraft_android_platform::pick_async("open", ""),
                        FileDialogRequest::OpenMany { .. } => artcraft_android_platform::pick_async("many", ""),
                        FileDialogRequest::Save { name, .. } => artcraft_android_platform::pick_async("save", &name),
                    }
                })),
                print: Some(Box::new(|bytes| artcraft_android_platform::print_pdf(bytes,"WordCraft"))),
                ..Default::default()
            };
            let mut editor = WordApp::new(session, services);
            if let Ok(state) = serde_json::from_str(&saved) {
                editor.apply_prefs(state);
            }
            editor.ui.language = "ru".into();
            editor.integrated_titlebar = false;
            editor.start_screen();
            Ok(Box::new(App {
                editor,
                prefs,
                saved,
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
