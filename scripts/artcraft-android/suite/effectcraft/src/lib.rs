use effectcraft_engine::Session;
use effectcraft_ui_egui as ui;
use eframe::egui;
#[path = "../../mobile_files.rs"]
mod mobile_files;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
struct App {
    editor: ui::EffectcraftApp,
    files: mobile_files::Files,
}
impl eframe::App for App {
    fn on_exit(&mut self) {
        self.editor.on_exit();
    }
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.editor.logic(ctx, frame);
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.editor.raw_input_hook(ctx, raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.editor.ui(ui, frame);
        let picked = egui::Window::new("Файлы на планшете")
            .default_open(false)
            .default_pos(egui::pos2(20.0, 65.0))
            .show(ui.ctx(), |ui| self.files.show(ui))
            .and_then(|r| r.inner)
            .flatten();
        if let Some((save, path)) = picked {
            let cmd = if save { "file.saveAs" } else { "file.open" };
            self.files.message = match self
                .editor
                .session
                .execute(cmd, serde_json::json!({"path":path}))
            {
                Ok(_) => if save {
                    "Документ сохранён"
                } else {
                    "Документ открыт"
                }
                .into(),
                Err(e) => format!("Не удалось выполнить действие: {e}"),
            };
        }
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
        PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.effectcraft.ru");
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
        "EffectCraft",
        options,
        Box::new(move |cc| {
            let mut editor = ui::EffectcraftApp::new(Session::new());
            editor.session.prefs.general.language = "ru".into();
            ui::theme::install(&cc.egui_ctx, &editor.tokens, "ru");
            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Композиция.ecproj"),
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
