//! Native Android host. No browser or network service is involved.
mod files;
mod recovery;
mod storage;

use eframe::{App, egui};
use photocraft_engine::Session;
use photocraft_ui_egui::{PhotocraftApp, Services};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

struct TabletApp {
    editor: PhotocraftApp,
    picker: Arc<Mutex<files::Picker>>,
}

impl TabletApp {
    fn new(cc: &eframe::CreationContext<'_>, private: PathBuf, documents: PathBuf) -> Self {
        let picker = Arc::new(Mutex::new(files::Picker::new(documents)));
        let pending = picker.clone();
        let prefs = private.join("preferences.json");
        let read_prefs = prefs.clone();
        let services = Services {
            import: Some(Box::new(|name, bytes, options| {
                storage::guard("открыть файл", || {
                    photocraft_io::import_with_options(name, bytes, options)
                        .map(|result| (result.document, result.warnings))
                        .map_err(|e| e.to_string())
                })
            })),
            export: Some(Box::new(|doc, name, settings| {
                storage::guard("сохранить файл", || {
                    photocraft_io::export(doc, name, &settings.io_options())
                        .map(|result| (result.bytes, result.warnings))
                        .map_err(|e| e.to_string())
                })
            })),
            file_dialog: Some(Box::new(move |request, _, reply| {
                pending
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .open(request, reply);
            })),
            write: Some(Box::new(|path, bytes| {
                storage::write_atomic(std::path::Path::new(path), bytes)
            })),
            file_stamp: Some(Box::new(|path| {
                let m = std::fs::metadata(path).ok()?;
                Some(photocraft_ui_egui::FileStamp {
                    len: m.len(),
                    modified: m.modified().ok(),
                })
            })),
            load_prefs: Some(Box::new(move || std::fs::read_to_string(&read_prefs).ok())),
            save_prefs: Some(Box::new(move |text| {
                storage::write_atomic(&prefs, text.as_bytes())
            })),
            ..recovery::services(private.join("recovery"))
        };
        let mut session = Session::new();
        session.print_spooler = Some(photocraft_engine::print_cmds::Spooler {
            name: "android",
            preferred: true,
            run: std::sync::Arc::new(|job| {
                artcraft_android_platform::print_pdf(&job.pdf, &job.title)?;
                Ok("Открыто окно печати Android".into())
            }),
        });
        session.edit_prefs(|prefs| prefs.interface.language = "ru".into());
        let mut editor = PhotocraftApp::new(session, services);
        editor.custom_titlebar = false;
        editor.integrated_titlebar = false;
        // Keep save errors synchronous: only mark a document saved after its bytes reach disk.
        editor.background_jobs = false;
        if let Some(render) = cc.wgpu_render_state.clone() {
            editor.set_wgpu(render);
        }
        Self { editor, picker }
    }
}

impl App for TabletApp {
    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.editor.logic(ctx, frame);
    }
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.editor.ui(ui, frame);
        self.picker
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .show(ui.ctx());
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, input: &mut egui::RawInput) {
        self.editor.raw_input_hook(ctx, input);
    }
    fn on_exit(&mut self) {
        self.editor.on_exit();
    }
}

// Android's loader resolves this exact symbol. This attribute exports a symbol;
// it does not dereference raw pointers or introduce an unsafe block.
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(android: winit::platform::android::activity::AndroidApp) {
    // SAFETY: AndroidApp supplies live JVM/activity handles for this process.
    if let Err(error) = unsafe { artcraft_android_platform::initialize(android.vm_as_ptr().cast(), android.activity_as_ptr().cast()) } {
        log::error!("{error}");
        return;
    }

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let Some(private) = android.internal_data_path() else {
        log::error!("Android did not provide the app data directory");
        return;
    };
    let documents = PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.photocraft.ru");
    let documents = match std::fs::create_dir_all(&documents) {
        Ok(()) => documents,
        Err(error) => {
            log::warn!("Shared app folder is unavailable: {error}");
            private.clone()
        }
    };
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..Default::default()
    };
    if let Err(error) = eframe::run_native(
        "PhotoCraft",
        options,
        Box::new(move |cc| Ok(Box::new(TabletApp::new(cc, private, documents)))),
    ) {
        log::error!("PhotoCraft could not start: {error}");
    }
}
