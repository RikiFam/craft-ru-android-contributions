use eframe::egui;
use gridcraft_engine::Session;
use gridcraft_ui_egui as ui;
#[path = "../../mobile_files.rs"]
mod mobile_files;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
struct App {
    editor: ui::SheetApp,
    files: mobile_files::Files,
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if let Some((id, params)) = artcraft_android_platform::replay() {
            if let Err(error) = self.editor.run(&id, params) {
                artcraft_android_platform::report(error);
            }
            artcraft_android_platform::finish_replay();
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
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
    let documents = PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.gridcraft.ru");
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
        "GridCraft",
        options,
        Box::new(move |_cc| {
            let mut editor = ui::SheetApp::new(Session::new(), ui::Services {
                pick_open: Some(Box::new(|| artcraft_android_platform::deferred_pick("open", "", "file.open"))),
                pick_save: Some(Box::new(|name| {
                    let name = if std::path::Path::new(name).extension().is_none() { format!("{name}.xlsx") } else { name.to_string() };
                    artcraft_android_platform::deferred_pick("save", &name, "file.saveAs")
                })),
                print: Some(Box::new(|pdf| {
                    if let Err(error) = artcraft_android_platform::print_pdf(pdf, "GridCraft") {
                        artcraft_android_platform::report(error);
                    }
                })),
                ..Default::default()
            });
            editor.ui.language = ui::i18n::Language::Ru;
            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Таблица.xlsx"),
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
