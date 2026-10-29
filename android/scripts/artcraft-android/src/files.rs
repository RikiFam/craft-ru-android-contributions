use eframe::egui;
use photocraft_ui_egui::{FileDialogAnswer, FileDialogReply, FileDialogRequest};
use std::path::PathBuf;
pub struct Picker;
impl Picker {
    pub fn new(_: PathBuf) -> Self { Self }
    pub fn open(&mut self, request: FileDialogRequest, reply: FileDialogReply) {
        let (kind, name) = match &request {
            FileDialogRequest::Save { suggested } => ("save", suggested.as_str()),
            FileDialogRequest::Folder { .. } => ("folder", ""),
            FileDialogRequest::Open { multiple, .. } => (if *multiple { "many" } else { "open" }, ""),
        };
        let result = artcraft_android_platform::pick_async(kind, name);
        std::thread::spawn(move || {
            let answer = match result.recv() {
                Ok(Ok(paths)) if paths.is_empty() => None,
                Ok(Ok(paths)) => Some(match request {
                    FileDialogRequest::Save { .. } => FileDialogAnswer::SaveTo(paths[0].clone()),
                    FileDialogRequest::Folder { .. } => FileDialogAnswer::FolderTo(paths[0].clone()),
                    _ => FileDialogAnswer::Paths(paths),
                }),
                Ok(Err(error)) => Some(FileDialogAnswer::Unavailable(error)),
                Err(error) => Some(FileDialogAnswer::Unavailable(error.to_string())),
            };
            reply.send(answer);
        });
    }
    pub fn show(&mut self, _: &egui::Context) {}
}
