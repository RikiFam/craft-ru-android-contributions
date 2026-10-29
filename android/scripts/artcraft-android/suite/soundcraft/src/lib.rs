use eframe::egui;
use soundcraft_engine::Engine;
use soundcraft_ui_egui as ui;
#[path = "../../mobile_files.rs"]
mod mobile_files;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
struct App {
    editor: ui::SoundApp,
    files: mobile_files::Files,
    path_request: Option<(String, String, std::sync::mpsc::Receiver<Result<Vec<String>, String>>)>,
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.path_request.is_none()
            && let Some(ui::dialogs::Dialog::PathPrompt { cmd, path, key, .. }) = &self.editor.dialogs.open
        {
            let kind = if key == "dir" { "folder" } else if cmd.starts_with("session.save") { "save" } else { "open" };
            let name = if kind == "save" && !path.to_lowercase().ends_with(".scraft") { "Запись.scraft" } else { path.as_str() };
            self.path_request = Some((cmd.clone(), key.clone(), artcraft_android_platform::pick_async(kind, name)));
            self.editor.dialogs.open = None;
        }
        if let Some((_, _, receiver)) = &self.path_request {
            match receiver.try_recv() {
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                result => {
                    if let Some((cmd, key, _)) = self.path_request.take() {
                        match result {
                            Ok(Ok(paths)) => if let Some(path) = paths.first() {
                                let mut params = serde_json::json!({});
                                params[key] = path.clone().into();
                                if let Err(error) = self.editor.run(&cmd, params) { self.files.message = error; }
                            },
                            Ok(Err(error)) => self.files.message = error,
                            Err(error) => self.files.message = error.to_string(),
                        }
                    }
                }
            }
        }
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
            let cmd = if save {
                "session.save_as"
            } else {
                "session.open"
            };
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
    let documents = PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.soundcraft.ru");
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
        "SoundCraft",
        options,
        Box::new(move |_cc| {
            let engine = Engine::default();
            let player =
                soundcraft_playback::Player::new(std::sync::Arc::new(engine.session().clone()));
            let mut editor = ui::SoundApp::new(engine, Some(player), ui::Services {
                pick_open: Some(Box::new(|cmd, _| artcraft_android_platform::deferred_pick("open", "", cmd).map(PathBuf::from))),
                pick_save: Some(Box::new(|cmd, name| artcraft_android_platform::deferred_pick("save", &name.to_string_lossy(), cmd).map(PathBuf::from))),
                ..Default::default()
            });

            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Запись.scraft"),
                path_request: None,
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
