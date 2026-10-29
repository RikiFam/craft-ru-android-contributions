use eframe::{App as _, egui};
use pdfcraft_ui_egui::PdfCraftApp;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
#[path = "../../mobile_files.rs"]
mod mobile_files;
#[path = "../../../src/storage.rs"]
mod storage;
struct App {
    editor: PdfCraftApp,
    files: mobile_files::Files,
    pending: Option<rfd::android::Request>,
    answer: Option<std::sync::mpsc::Receiver<Result<Vec<String>, String>>>,
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.editor.logic(ctx, frame);
        if self.pending.is_none() {
            if let Some(request) = rfd::android::take_request() {
                let kind = match request.kind {
                    rfd::android::Kind::Save => "save",
                    rfd::android::Kind::Folder => "folder",
                    rfd::android::Kind::OpenMany => "many",
                    rfd::android::Kind::Open => "open",
                };
                self.answer = Some(artcraft_android_platform::pick_async(kind, &request.name));
                self.pending = Some(request);
            }
        }
        if let Some(answer) = &self.answer {
            match answer.try_recv() {
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
                result => {
                    if let Some(request) = self.pending.take() {
                        match result {
                            Ok(Ok(paths)) => request.finish(Some(paths.into_iter().map(PathBuf::from).collect())),
                            Ok(Err(error)) => { artcraft_android_platform::report(error); request.finish(None); }
                            Err(error) => { artcraft_android_platform::report(error.to_string()); request.finish(None); }
                        }
                    }
                    self.answer = None;
                }
            }
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.editor.raw_input_hook(ctx, raw);
    }
    fn on_exit(&mut self) {
        self.editor.on_exit();
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.editor.ui(ui, frame);
        let picked = if self.pending.is_some() {
            None
        } else {
            egui::Window::new("Файлы на планшете")
                .default_open(false)
                .default_pos(egui::pos2(20.0, 65.0))
                .show(ui.ctx(), |ui| self.files.show(ui))
                .and_then(|r| r.inner)
                .flatten()
        };
        if let Some((save, path)) = picked {
            if let Some(request) = self.pending.take() {
                request.finish(Some(vec![PathBuf::from(path)]));
            } else if save {
                self.files.message = match self.editor.active_ids() {
                    Some((_, id)) => match self.editor.session.save_bytes(id) {
                        Ok(bytes) => {
                            match storage::write_atomic(std::path::Path::new(&path), bytes.as_ref())
                            {
                                Ok(()) => "Копия PDF сохранена".into(),
                                Err(e) => format!("Не удалось сохранить: {e}"),
                            }
                        }
                        Err(e) => format!("Не удалось сохранить: {e}"),
                    },
                    None => "Сначала откройте документ".into(),
                };
            } else {
                self.editor.open_path(&path);
            }
        }
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
    let documents = PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.pdfcraft.ru");
    let home = if std::fs::create_dir_all(&documents).is_ok() {
        documents
    } else {
        private
    };
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "PdfCraft",
        options,
        Box::new(move |cc| {
            pdfcraft_ui_egui::theme::install_fonts(&cc.egui_ctx);
            let mut editor = PdfCraftApp::new();
            editor.language = "ru".into();
            editor.system_clipboard = false;
            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Документ.pdf"),
                pending: None,
                answer: None,
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
