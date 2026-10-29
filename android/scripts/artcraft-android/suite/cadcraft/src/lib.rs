use cadcraft_engine::Session;
use cadcraft_ui_egui as ui;
use eframe::egui;
#[path = "../../mobile_files.rs"]
mod mobile_files;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
struct App {
    editor: ui::CadApp,
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
            let cmd = if save { "saveas" } else { "open" };
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
    let documents = PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.cadcraft.ru");
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
        "CADCraft",
        options,
        Box::new(move |_cc| {
            cadcraft_engine::cmd::file::set_io(cadcraft_engine::cmd::file::IoHooks {
                read: |bytes, name| cadcraft_io::read(bytes, name).map_err(|e| e.to_string()),
                write: |doc, name| cadcraft_io::write(doc, name).map_err(|e| e.to_string()),
                plot: Some(|doc, space, options| cadcraft_io::plotter::plot(doc, space, options).map_err(|e| e.to_string())),
            });
            cadcraft_engine::cmd::file::set_write_as(|doc, name, options| {
                cadcraft_io::write_with(doc, name, &cadcraft_io::SaveOptions { version: options.version, binary: options.binary }).map_err(|e| e.to_string())
            });
            cadcraft_engine::cmd::file::set_entity_codec(cadcraft_engine::cmd::file::EntityCodec {
                groups: cadcraft_io::entity_groups,
                entity: cadcraft_io::entity_from_groups,
            });
            let mut editor = ui::CadApp::new(Session::new(), ui::Services {
                pick_open: Some(Box::new(|| artcraft_android_platform::deferred_pick("open", "", "open"))),
                pick_save: Some(Box::new(|name| artcraft_android_platform::deferred_pick("save", name, "saveas"))),
                pick_lisp: Some(Box::new(|_| artcraft_android_platform::deferred_pick("open", "", "appload"))),
                ..Default::default()
            });
            editor.ui.interface_language = ui::i18n::Preference::Russian;
            editor.integrated_titlebar = false;
            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Чертёж.dxf"),
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
