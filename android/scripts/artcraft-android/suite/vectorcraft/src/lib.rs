#[path = "../../../src/storage.rs"]
mod storage;
use eframe::egui;
use vectorcraft_engine::Session;
use vectorcraft_ui_egui as ui;
#[path = "../../mobile_files.rs"]
mod mobile_files;
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;
struct AndroidPrint;
impl ui::print::PrintService for AndroidPrint {
    fn printers(&mut self) -> Vec<ui::print::Printer> {
        vec![ui::print::Printer{name:"Системное окно Android".into(),default:true}]
    }
    fn print(&mut self, job: &ui::print::PrintJob) -> Result<String,String> {
        artcraft_android_platform::print_pdf(job.pdf,job.title)?;
        Ok("Открыто окно печати Android".into())
    }
}
struct App {
    editor: ui::VectorcraftApp,
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
    let documents =
        PathBuf::from("/storage/emulated/0/Android/media/ai.storyteller.vectorcraft.ru");
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
        "VectorCraft",
        options,
        Box::new(move |_cc| {
            let mut editor = ui::VectorcraftApp::new(
                Session::new(),
                ui::Services {
                    print: Some(Box::new(AndroidPrint)),
                    start_pick: Some(Box::new(|request| {
                        use ui::picks::PickRequest;
                        Some(match request {
                            PickRequest::Open(_) => artcraft_android_platform::pick_paths("open", ""),
                            PickRequest::OpenMany => artcraft_android_platform::pick_paths("many", ""),
                            PickRequest::Save(pick) => artcraft_android_platform::pick_paths("save", &pick.name),
                            PickRequest::Folder => artcraft_android_platform::pick_paths("folder", ""),
                        })
                    })),
                    read: Some(Box::new(|p| std::fs::read(p).map_err(|e| e.to_string()))),
                    write: Some(Box::new(|p, b| {
                        storage::write_atomic(std::path::Path::new(p), b)
                    })),
                    ..Default::default()
                },
            );
            editor.session.prefs.interface_language = "ru".into();
            editor.custom_titlebar = false;
            Ok(Box::new(App {
                editor,
                files: mobile_files::Files::new(home, "Рисунок.svg"),
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
