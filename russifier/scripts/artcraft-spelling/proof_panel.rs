//! Russian proofing for editors without a built-in spelling dialog.
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct Texts { pub items: Vec<(String, String)>, pub truncated: bool, pub scope: String, bytes: usize }
impl Texts {
    pub fn new(scope: &str) -> Self { Self { scope: scope.into(), ..Self::default() } }
    pub fn push(&mut self, label: impl Into<String>, text: &str) {
        if text.trim().is_empty() { return; }
        if self.items.len() >= 2000 || self.bytes.saturating_add(text.len()) > 1_000_000 { self.truncated = true; return; }
        self.bytes += text.len(); self.items.push((label.into(), text.to_owned()));
    }
}
#[allow(dead_code)]
pub fn strings(texts: &mut Texts, label: &str, value: &serde_json::Value, depth: usize) {
    if depth > 8 { texts.truncated = true; return; }
    match value {
        serde_json::Value::String(s) => texts.push(label, s),
        serde_json::Value::Array(a) => { texts.truncated |= a.len()>2000; for v in a.iter().take(2000) { strings(texts, label, v, depth+1); } },
        serde_json::Value::Object(o) => { texts.truncated |= o.len()>2000; for v in o.values().take(2000) { strings(texts, label, v, depth+1); } },
        _ => {}
    }
}
#[derive(Clone)]
struct Issue { item: usize, start: usize, end: usize, word: String }
#[derive(Default)]
struct State { open: bool, refresh: bool, texts: Texts, issues: Vec<Issue>, status: String, busy: bool, selected: Option<usize>, suggestions: Vec<String>, generation: u64 }

fn inspect(texts: &Texts) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut checked = std::collections::HashMap::new();
    for (item, (_, text)) in texts.items.iter().enumerate() {
        let mut start = None;
        for (at, c) in text.char_indices().chain(std::iter::once((text.len(), ' '))) {
            if c.is_alphabetic() { start.get_or_insert(at); }
            else if let Some(from) = start.take() {
                let Some(word) = text.get(from..at) else { continue };
                if !crate::russian_spell::is_russian(word) || word.chars().count() < 2 { continue; }
                let good = *checked.entry(word.to_owned()).or_insert_with(|| crate::russian_spell::russian().check(word));
                if !good { issues.push(Issue { item, start: from, end: at, word: word.to_owned() }); }
                if issues.len() >= 500 { return issues; }
            }
        }
    }
    issues
}

// Separate selection and source scopes keep this UI branch readable.
#[allow(clippy::collapsible_if)]
pub fn show(ui: &mut egui::Ui, get_texts: impl FnOnce() -> Result<Texts, String>) {
    let ctx = ui.ctx().clone();
    let id = egui::Id::unique("artcraft-russian-proof");
    let shared = ctx.data_mut(|d| d.get_temp_mut_or_default::<Arc<Mutex<State>>>(id).clone());
    let mut request = {
        let mut state=shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        std::mem::take(&mut state.refresh)
    };
    let panel = if cfg!(target_os = "android") {
        egui::Panel::top("artcraft-russian-proof-toolbar")
    } else {
        egui::Panel::bottom("artcraft-russian-proof-toolbar")
    };
    panel.show(ui, |ui| {
        if ui.button("Орфография · русский").clicked() { request = true; }
    });
    if request {
        let result = get_texts();
        let mut s = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        s.open = true;
        if !s.busy {
            s.generation = s.generation.wrapping_add(1); s.issues.clear(); s.selected = None; s.suggestions.clear();
            match result {
                Ok(texts) => { s.texts = texts; s.status = "Проверка…".into(); s.busy = true; }
                Err(error) => { s.status = error; s.texts = Texts::default(); }
            }
            if s.busy {
                let texts = s.texts.clone(); let generation = s.generation; let result_state = shared.clone(); let repaint = ctx.clone();
                let work = move || {
                    let issues = inspect(&texts);
                    let mut state = result_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    if state.generation == generation {
                        state.status = if texts.items.is_empty() { "Нет доступного текста. Выберите текст или откройте проект.".into() }
                        else if texts.truncated || issues.len() == 500 { "Проверена только часть текста: достигнут лимит. Разбейте большой проект на части.".into() }
                        else if issues.is_empty() { "Орфографических ошибок в русском тексте не найдено.".into() }
                        else { format!("Возможных ошибок: {}", issues.len()) };
                        state.issues = issues; state.busy = false;
                    }
                    repaint.request_repaint();
                };
                #[cfg(not(target_arch = "wasm32"))]
                if let Err(error) = std::thread::Builder::new().name("russian-proof".into()).spawn(work) { s.busy = false; s.status = format!("Не удалось начать проверку: {error}"); }
                #[cfg(target_arch = "wasm32")]
                { drop(s); work(); }
            }
        }
    }
    let mut state = shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if !state.open { return; }
    let mut open = true;
    egui::Window::new("Проверка орфографии — русский").open(&mut open).default_size([640.0, 460.0]).show(&ctx, |ui| {
        ui.label(&state.status);
        ui.label(&state.texts.scope);
        if ui.add_enabled(!state.busy, egui::Button::new("Проверить заново")).clicked() { state.refresh=true; ctx.request_repaint(); }
        ui.label("Проверяется текст проекта, доступный этому редактору. Названия и специальные термины могут отсутствовать в словаре.");
        ui.separator();
        egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
            let mut select = None;
            for (i, issue) in state.issues.iter().enumerate() {
                let label = state.texts.items.get(issue.item).map(|x| x.0.as_str()).unwrap_or("");
                if ui.selectable_label(state.selected == Some(i), format!("{} — {}", issue.word, label)).clicked() { select = Some(i); }
            }
            if let Some(i) = select {
                state.selected = Some(i);
                state.suggestions = state.issues.get(i).map(|issue| crate::russian_spell::russian().suggest(&issue.word, 6)).unwrap_or_default();
            }
        });
        if let Some(issue) = state.selected.and_then(|i| state.issues.get(i)).cloned() {
            if let Some((_, text)) = state.texts.items.get(issue.item) {
                ui.separator();
                let from = text.get(..issue.start).unwrap_or("").char_indices().rev().nth(35).map(|(i,_)| i).unwrap_or(0);
                let to = text.get(issue.end..).unwrap_or("").char_indices().nth(50).map(|(i,_)| issue.end+i).unwrap_or(text.len());
                ui.label(text.get(from..to).unwrap_or(""));
                ui.label("Варианты замены (нажатие копирует исправленный фрагмент):");
                for suggestion in &state.suggestions {
                    if ui.button(suggestion).clicked() {
                        let mut corrected = text.clone();
                        if corrected.get(issue.start..issue.end) == Some(issue.word.as_str()) {
                            corrected.replace_range(issue.start..issue.end, suggestion);
                            #[cfg(target_os = "android")]
                            if let Err(error) = artcraft_android_platform::copy_proof_text(&corrected) { artcraft_android_platform::report(error); }
                            ctx.copy_text(corrected);
                        }
                    }
                }
                if state.suggestions.is_empty() { ui.label("Подходящих вариантов не найдено."); }
                ui.label("Сам проект не изменяется. Вставьте исправление в нужное текстовое поле.");
            }
        }
    });
    state.open = open;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn russian_review_reports_typo_with_valid_utf8_range() {
        let mut texts=Texts::default(); texts.push("Тест", "Документами привет превет hello.");
        let issues=inspect(&texts);
        assert_eq!(issues.len(),1); assert_eq!(issues[0].word,"превет");
        assert_eq!(texts.items[0].1.get(issues[0].start..issues[0].end),Some("превет"));
    }
    #[test]
    fn oversized_input_is_reported_as_partial() {
        let mut texts=Texts::default(); texts.push("Тест", &"я".repeat(600_000));
        assert!(texts.truncated); assert!(texts.items.is_empty());
    }
}
