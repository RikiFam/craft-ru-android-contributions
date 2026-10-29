use crate::storage::save_path;
use eframe::egui;
use photocraft_ui_egui::{FileDialogAnswer, FileDialogReply, FileDialogRequest};
use std::path::{Path, PathBuf};

pub struct Picker {
    home: PathBuf,
    directory: PathBuf,
    pending: Option<(FileDialogRequest, FileDialogReply)>,
    name: String,
    error: Option<String>,
}

impl Picker {
    pub fn new(home: PathBuf) -> Self {
        Self {
            directory: home.clone(),
            home,
            pending: None,
            name: String::new(),
            error: None,
        }
    }
    pub fn open(&mut self, request: FileDialogRequest, reply: FileDialogReply) {
        self.name = match &request {
            FileDialogRequest::Save { suggested } => Path::new(suggested)
                .file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("Изображение.psd")
                .to_owned(),
            _ => String::new(),
        };
        self.error = None;
        self.pending = Some((request, reply));
    }
    pub fn show(&mut self, ctx: &egui::Context) {
        let Some((request, _)) = &self.pending else {
            return;
        };
        let save = matches!(request, FileDialogRequest::Save { .. });
        let folder = matches!(request, FileDialogRequest::Folder { .. });
        let mut answer = None;
        let mut cancel = false;
        egui::Modal::new(egui::Id::new("android-file-picker")).show(ctx, |ui| {
            ui.set_min_width(440.0_f32.min(ctx.content_rect().width() - 60.0));
            ui.heading(if save {
                "Сохранить файл"
            } else if folder {
                "Выбрать папку"
            } else {
                "Открыть файл"
            });
            ui.label("Рабочая папка PhotoCraft");
            ui.small(self.directory.to_string_lossy());
            ui.label("Файлы можно скопировать сюда через приложение «Мои файлы» или USB.");
            ui.horizontal(|ui| {
                if ui.button("В рабочую папку").clicked() {
                    self.directory = self.home.clone();
                }
                if self.directory != self.home
                    && ui.button("На уровень выше").clicked()
                    && let Some(parent) = self.directory.parent()
                {
                    self.directory = parent.to_path_buf();
                }
            });
            let entries = std::fs::read_dir(&self.directory).map(|entries| {
                let mut entries: Vec<_> = entries
                    .filter_map(Result::ok)
                    .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
                    .collect();
                entries.sort_by_key(|e| (!e.path().is_dir(), e.file_name()));
                entries
            });
            match entries {
                Ok(entries) => {
                    egui::ScrollArea::vertical()
                        .max_height(280.0)
                        .show(ui, |ui| {
                            for entry in entries {
                                let path = entry.path();
                                let dir = path.is_dir();
                                let label = format!(
                                    "{}{}",
                                    if dir { "▸ " } else { "" },
                                    entry.file_name().to_string_lossy()
                                );
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 36.0],
                                        egui::Button::new(label),
                                    )
                                    .clicked()
                                {
                                    if dir {
                                        self.directory = path;
                                    } else if save {
                                        self.name =
                                            entry.file_name().to_string_lossy().into_owned();
                                    } else if !folder {
                                        answer = Some(FileDialogAnswer::Paths(vec![
                                            path.to_string_lossy().into_owned(),
                                        ]));
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
                ui.label("Имя и расширение файла, например Рисунок.png:");
                ui.text_edit_singleline(&mut self.name);
            }
            if let Some(error) = &self.error {
                ui.label(error);
            }
            ui.horizontal(|ui| {
                cancel = ui.button("Отмена").clicked();
                if save && ui.button("Сохранить").clicked() {
                    match save_path(&self.directory, &self.name) {
                        Ok(path) => {
                            answer = Some(FileDialogAnswer::SaveTo(
                                path.to_string_lossy().into_owned(),
                            ))
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                if folder && ui.button("Выбрать эту папку").clicked() {
                    answer = Some(FileDialogAnswer::FolderTo(
                        self.directory.to_string_lossy().into_owned(),
                    ));
                }
            });
        });
        if (cancel || answer.is_some())
            && let Some((_, reply)) = self.pending.take()
        {
            reply.send(answer);
        }
    }
}
