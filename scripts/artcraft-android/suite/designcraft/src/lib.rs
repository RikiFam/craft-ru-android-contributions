#[path = "../../../src/storage.rs"]
mod storage;
use designcraft_engine::Session;
use designcraft_ui_egui as ui;
use eframe::egui;
#[path = "../../mobile_files.rs"]
mod mobile_files;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
struct App {
    editor: ui::DesignApp,
    files: mobile_files::Files,
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.editor.logic(ctx);
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.editor.raw_input_hook(ctx, raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if let Some((save, path)) = self.files.show(ui) {
            let cmd = if save { "file.saveAs" } else { "file.open" };
            self.files.message = match self.editor.run(cmd, serde_json::json!({"path":path})) {
                Ok(_) => if save {
                    "Документ сохранён"
                } else {
                    "Документ открыт"
                }
                .into(),
                Err(e) => format!("Не удалось выполнить действие: {e}"),
            };
        }
        self.editor.ui(ui);
    }
}
#[unsafe(no_mangle)]
fn android_main(android: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let Some(private) = android.internal_data_path() else {
        return;
    };
    let documents =
        PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.designcraft.ru");
    let home = if std::fs::create_dir_all(&documents).is_ok() {
        documents
    } else {
        private
    };
    if let Err(e) = std::env::set_current_dir(&home) {
        log::warn!("{e}");
    }
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "DesignCraft",
        options,
        Box::new(move |_cc| {
            let mut editor = ui::DesignApp::new(
                Session::new(),
                ui::Services {
                    read: Some(Box::new(|p| std::fs::read(p).map_err(|e| e.to_string()))),
                    write: Some(Box::new(|p, b| {
                        storage::write_atomic(std::path::Path::new(p), b)
                    })),
                    ..Default::default()
                },
            );
            editor.ui.language = "ru".into();
            editor.integrated_titlebar = false;
            editor.custom_titlebar = false;
            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Макет.designcraft"),
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
