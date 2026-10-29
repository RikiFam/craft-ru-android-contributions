use eframe::egui;
use figcraft_engine::Session;
use figcraft_ui_egui as ui;
#[path = "../../mobile_files.rs"]
mod mobile_files;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
struct App {
    editor: ui::FigApp,
    files: mobile_files::Files,
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.editor.logic(ctx);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if let Some((save, path)) = self.files.show(ui) {
            let cmd = if save { "file.saveAs" } else { "file.open" };
            self.files.message = match self.editor.run(cmd, serde_json::json!({"path":path})) {
                Some(_) => if save {
                    "Документ сохранён"
                } else {
                    "Документ открыт"
                }
                .into(),
                None => "Не удалось выполнить действие; подробности в уведомлении редактора".into(),
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
    let documents = PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.figcraft.ru");
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
        "FigCraft",
        options,
        Box::new(move |cc| {
            let mut editor = ui::FigApp::new(Session::new(), Box::new(ui::NoServices));
            editor.session.prefs.language = "ru".into();
            editor.logic(&cc.egui_ctx);
            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Дизайн.figcraft"),
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
