use eframe::egui;
use std::path::PathBuf;

pub struct Files {
    root: PathBuf,
    directory: PathBuf,
    mode: Option<bool>,
    name: String,
    confirm: bool,
    folder: bool,
    pub message: String,
}
impl Files {
    pub fn new(root: PathBuf, name: &str) -> Self {
        Self {
            directory: root.clone(),
            root,
            mode: None,
            name: name.into(),
            confirm: false,
            folder: false,
            message: String::new(),
        }
    }
    pub fn begin(&mut self, save: bool, folder: bool, name: &str) {
        self.mode = Some(save);
        self.folder = folder;
        self.confirm = false;
        if !name.is_empty() {
            self.name = PathBuf::from(name)
                .file_name()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
        }
    }
    pub fn is_open(&self) -> bool {
        self.mode.is_some()
    }
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<(bool, String)> {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(28, 31, 36))
            .show(ui, |ui| {
                ui.style_mut().visuals = egui::Visuals::dark();
                ui.horizontal(|ui| {
                    ui.set_min_height(30.0);
                    ui.label("Файлы на планшете:");
                    if ui.button("Открыть…").clicked() {
                        self.begin(false, false, "");
                    }
                    if ui.button("Сохранить как…").clicked() {
                        self.begin(true, false, "");
                    }
                    ui.label(&self.message);
                });
            });
        self.show_picker(ui.ctx())
    }
    pub fn show_picker(&mut self, ctx: &egui::Context) -> Option<(bool, String)> {
        let save = self.mode?;
        let mut result = None;
        let mut cancel = false;
        egui::Modal::new(egui::Id::unique("tablet-files")).show(ctx, |ui| {
            ui.set_width(540.0_f32.min(ui.ctx().content_rect().width() - 60.0));
            ui.heading(if save {
                "Сохранить документ"
            } else {
                "Открыть документ"
            });
            ui.label(self.directory.to_string_lossy());
            if self.directory != self.root && ui.button("На уровень выше").clicked() {
                if let Some(parent) = self.directory.parent() {
                    self.directory = parent.to_owned();
                }
            }
            match std::fs::read_dir(&self.directory) {
                Ok(entries) => {
                    let mut entries: Vec<_> = entries.take(5000).filter_map(Result::ok).collect();
                    entries.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));
                    egui::ScrollArea::vertical()
                        .max_height(320.0)
                        .show(ui, |ui| {
                            for entry in entries {
                                let path = entry.path();
                                let name = entry.file_name().to_string_lossy().into_owned();
                                if ui
                                    .button(if path.is_dir() {
                                        format!("Папка: {name}")
                                    } else {
                                        name.clone()
                                    })
                                    .clicked()
                                {
                                    if path.is_dir() {
                                        self.directory = path;
                                    } else if save {
                                        self.name = name;
                                        self.confirm = false;
                                    } else if !self.folder {
                                        result = Some((false, path.to_string_lossy().into_owned()));
                                    }
                                }
                            }
                        });
                }
                Err(e) => {
                    ui.label(format!("Не удалось прочитать папку: {e}"));
                }
            }
            if save {
                if ui.text_edit_singleline(&mut self.name).changed() {
                    self.confirm = false;
                }
                let name = self.name.trim();
                let valid = !name.is_empty()
                    && name != "."
                    && name != ".."
                    && !name.contains(['/', '\\', '\0']);
                let path = self.directory.join(name);
                if self.confirm {
                    ui.label("Файл уже существует. Заменить его?");
                }
                if ui
                    .add_enabled(
                        valid,
                        egui::Button::new(if self.confirm {
                            "Заменить файл"
                        } else {
                            "Сохранить"
                        }),
                    )
                    .clicked()
                {
                    if path.exists() && !self.confirm {
                        self.confirm = true;
                    } else {
                        result = Some((true, path.to_string_lossy().into_owned()));
                    }
                }
            }
            if self.folder && ui.button("Выбрать эту папку").clicked() {
                result = Some((false, self.directory.to_string_lossy().into_owned()));
            }
            if ui.button("Отмена").clicked() {
                cancel = true;
            }
        });
        if result.is_some() || cancel {
            self.mode = None;
        }
        result
    }
}
