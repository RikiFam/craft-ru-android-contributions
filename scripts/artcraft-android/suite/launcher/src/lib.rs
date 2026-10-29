use artcraft_launcher_core::protocol::{
    Backend, Capabilities, Command, Event, LatestRelease, Origin,
};
use artcraft_launcher_core::status::InstallSource;
use artcraft_launcher_core::{Installed, Platform, SUITE};
use artcraft_launcher_ui_egui::{Host, LauncherUi, Settings};
use eframe::egui;
use jni::{
    JavaVM,
    objects::{GlobalRef, JObject, JValue},
};
use std::path::PathBuf;
use winit::platform::android::activity::AndroidApp;

struct AndroidBackend {
    vm: JavaVM,
    activity: GlobalRef,
    events: Vec<Event>,
}
impl AndroidBackend {
    fn new(android: &AndroidApp) -> Result<Self, String> {
        // SAFETY: AndroidApp owns these valid VM/global activity pointers throughout
        // android_main. A new global reference is retained before the borrowed view ends.
        let vm =
            unsafe { JavaVM::from_raw(android.vm_as_ptr().cast()) }.map_err(|e| e.to_string())?;
        let activity = {
            let env = vm.attach_current_thread().map_err(|e| e.to_string())?;
            let borrowed = unsafe { JObject::from_raw(android.activity_as_ptr().cast()) };
            env.new_global_ref(&borrowed).map_err(|e| e.to_string())?
        };
        Ok(Self {
            vm,
            activity,
            events: Vec::new(),
        })
    }
    fn package(&self, app: &str, launch: bool) -> Result<bool, String> {
        if !SUITE.iter().any(|a| a.id == app) {
            return Err("Неизвестное приложение".into());
        }
        let mut env = self.vm.attach_current_thread().map_err(|e| e.to_string())?;
        let result = env.with_local_frame(16, |env| -> jni::errors::Result<bool> {
            let manager = env
                .call_method(
                    self.activity.as_obj(),
                    "getPackageManager",
                    "()Landroid/content/pm/PackageManager;",
                    &[],
                )?
                .l()?;
            let name = env.new_string(format!("ai.storyteller.{app}.ru"))?;
            let intent = env
                .call_method(
                    manager,
                    "getLaunchIntentForPackage",
                    "(Ljava/lang/String;)Landroid/content/Intent;",
                    &[JValue::Object(&name)],
                )?
                .l()?;
            if intent.is_null() {
                return Ok(false);
            }
            if launch {
                env.call_method(
                    self.activity.as_obj(),
                    "startActivity",
                    "(Landroid/content/Intent;)V",
                    &[JValue::Object(&intent)],
                )?;
            }
            Ok(true)
        });
        if result.is_err() {
            let _ = env.exception_clear();
        }
        result.map_err(|e| format!("Не удалось открыть приложение: {e}"))
    }
    fn detect(&mut self) {
        for app in SUITE {
            let installed = match self.package(app.id, false) {
                Ok(true) => Some(Installed {
                    version: None,
                    source: InstallSource::Binary,
                    location: format!("ai.storyteller.{}.ru", app.id),
                    package: None,
                }),
                Ok(false) => None,
                Err(error) => {
                    log::error!("{error}");
                    None
                }
            };
            self.events.push(Event::Detected {
                app: app.id.into(),
                installed,
            });
            self.events.push(Event::Latest {
                app: app.id.into(),
                latest: LatestRelease::None,
                from: Origin::ArtCraftApi,
            });
        }
    }
}
impl Backend for AndroidBackend {
    fn send(&mut self, command: Command) {
        match command {
            Command::Detect | Command::CheckForUpdates { .. } => self.detect(),
            Command::Launch { app } => {
                let result = self.package(&app, true).and_then(|found| if found { Ok("Приложение открыто".into()) } else { Err("Приложение ещё не установлено".into()) });
                self.events.push(Event::JobFinished { app, result });
            }
            Command::FetchScreenshots { .. } | Command::Configure(_) => {},
            Command::Install { app } | Command::Replace { app } | Command::Uninstall { app } | Command::Reveal { app } => self.events.push(Event::JobFinished { app, result: Err("Установка и удаление выполняются средствами Android. Сборки для Windows сюда не подходят.".into()) }),
            _ => {}
        }
    }
    fn poll(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            manage_apps: true,
            reveal_label: "Сведения о приложении",
            self_update: false,
        }
    }
    fn platform(&self) -> Option<Platform> {
        None
    }
    fn now(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }
}
struct AndroidHost;
impl Host for AndroidHost {}
struct App {
    launcher: LauncherUi,
    prefs: PathBuf,
    saved: String,
}
impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.launcher.logic(ctx, &mut AndroidHost);
    }
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        self.launcher.raw_input(ctx, raw);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.launcher.ui(ui, &mut AndroidHost);
        if let Ok(text) = serde_json::to_string(&self.launcher.settings) {
            if text != self.saved && std::fs::write(&self.prefs, &text).is_ok() {
                self.saved = text;
            }
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
    let backend = match AndroidBackend::new(&android) {
        Ok(b) => b,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };
    let prefs = private.join("settings.json");
    let saved = std::fs::read_to_string(&prefs).unwrap_or_default();
    let mut settings: Settings = serde_json::from_str(&saved).unwrap_or_default();
    settings.language = "ru".into();
    settings.close_to_tray = false;
    let options = eframe::NativeOptions {
        android_app: Some(android),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native(
        "ArtCraft Launcher",
        options,
        Box::new(move |cc| {
            Ok(Box::new(App {
                launcher: LauncherUi::new(&cc.egui_ctx, Box::new(backend), settings),
                prefs,
                saved,
            }))
        }),
    ) {
        log::error!("{e}");
    }
}
