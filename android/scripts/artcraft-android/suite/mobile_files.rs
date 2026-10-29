use eframe::egui;
use std::{path::PathBuf, sync::mpsc::{Receiver, TryRecvError}};
pub struct Files {
    pending: Option<Receiver<Result<Vec<String>,String>>>,
    save: bool,
    name: String,
    pub message: String,
}
impl Files {
    pub fn new(_: PathBuf, name: &str) -> Self {
        Self { pending: None, save: false, name: name.into(), message: String::new() }
    }
    pub fn begin(&mut self, save: bool, folder: bool, name: &str) {
        if self.pending.is_some() { return; }
        if !name.is_empty() { self.name=name.into(); }
        self.save=save;
        self.pending=Some(artcraft_android_platform::pick_async(if folder {"folder"} else if save {"save"} else {"open"},&self.name));
    }
    pub fn is_open(&self) -> bool { self.pending.is_some() }
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<(bool,String)> {
        for error in artcraft_android_platform::take_errors() { self.message=error; }
        ui.horizontal(|ui| {
            if ui.button("Открыть…").clicked() { self.begin(false,false,""); }
            if ui.button("Сохранить как…").clicked() { self.begin(true,false,""); }
            ui.label(&self.message);
        });
        self.show_picker(ui.ctx())
    }
    pub fn show_picker(&mut self, ctx: &egui::Context) -> Option<(bool,String)> {
        let result=self.pending.as_ref()?.try_recv();
        ctx.request_repaint_after(std::time::Duration::from_millis(50));
        match result {
            Err(TryRecvError::Empty) => None,
            other => {
                self.pending=None;
                match other {
                    Ok(Ok(paths)) => paths.into_iter().next().map(|p|(self.save,p)),
                    Ok(Err(error)) => { self.message=error; None },
                    Err(error) => { self.message=error.to_string(); None },
                }
            }
        }
    }
}
