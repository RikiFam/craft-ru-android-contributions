use eframe::egui;
#[path = "../../../src/storage.rs"]
mod storage;
use lightcraft_engine::Session;
use lightcraft_ui_egui as ui;
#[path = "../../mobile_files.rs"]
mod mobile_files;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
struct App {
    editor: ui::LightcraftApp,
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
            let cmd = if save { "app.export" } else { "library.import" };
            self.files.message = match self.editor.run(
                cmd,
                if save {
                    serde_json::json!({"path":path})
                } else {
                    serde_json::json!({"paths":[path]})
                },
            ) {
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
    let documents = PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.lightcraft.ru");
    let home = if std::fs::create_dir_all(&documents).is_ok() {
        documents
    } else {
        private.clone()
    };
    if let Err(e) = std::env::set_current_dir(&home) {
        log::warn!("{e}");
    }
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "LightCraft",
        options,
        Box::new(move |_cc| {
            let mut session = Session::new();
            session
                .open_library(private.join("library"), false)
                .map_err(|e| e.to_string())?;
            session.auto_sync = true;
            let mut editor = ui::LightcraftApp::new(
                session,
                ui::Services {
                    write: Some(Box::new(|p, b| {
                        storage::write_atomic(std::path::Path::new(p), b)
                    })),
                    write_shared: Some(std::sync::Arc::new(|p, b| {
                        storage::write_atomic(std::path::Path::new(p), b)
                    })),
                    ..Default::default()
                },
            );
            editor.ui.language = ui::i18n::Locale::Ru;
            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Фото.jpg"),
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
