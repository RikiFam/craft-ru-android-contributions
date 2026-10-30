"""Add an offline Russian review panel to editors without an existing checker."""
from prepare import ROOT,HERE,edit,module
import re

SCOPES={
 'vectorcraft':'Текстовые объекты документа.',
 'cadcraft':'Однострочные и многострочные надписи в пространстве модели.',
 'figcraft':'Текстовые объекты макета.',
 'pdfcraft':'Выделенный текст PDF.',
 'lightcraft':'Названия, подписи и описания выбранных фотографий.',
 'filmcraft':'Субтитры, титры и текстовые сведения о материалах проекта.',
 'effectcraft':'Текстовые слои активной композиции.',
 'soundcraft':'Названия и комментарии проекта, дорожек и маркеров.',
}

READERS={
'vectorcraft':r'''
let doc = self.session.doc().map_err(|e| e.to_string())?;
let mut pending: Vec<_> = doc.doc.layers.iter().map(|n| n.as_ref()).collect();
let mut visited = 0;
while let Some(node) = pending.pop() {
    visited += 1; if visited > 100_000 { texts.truncated = true; break; }
    if let vectorcraft_doc::NodeKind::Text(t) = &node.kind {
        let text: String = t.runs.iter().map(|r| r.text.as_str()).collect();
        texts.push(node.name.clone().unwrap_or_else(|| "Текстовый объект".into()), &text);
    }
    if let Some(children) = node.children() { pending.extend(children.iter().map(|n| n.as_ref())); }
}
''',
'cadcraft':r'''
let doc = self.session.doc().map_err(|e| e.to_string())?;
for entity in doc.model.iter().take(100_000) {
    match &entity.kind {
        cadcraft_doc::EntityKind::Text(t) => texts.push("Надпись в чертеже", &t.value),
        cadcraft_doc::EntityKind::MText(t) => texts.push("Многострочный текст", &t.contents),
        _ => {}
    }
}
''',
'figcraft':r'''
let doc = self.session.doc().map_err(|e| e.to_string())?;
for node in doc.nodes().take(100_000) {
    if let figcraft_doc::NodeKind::Text(t) = &node.kind { texts.push(node.name.clone(), &t.characters); }
}
''',
'pdfcraft':r'''
let view = self.active.and_then(|i| self.views.get(i)).ok_or_else(|| "Сначала откройте PDF и выделите текст для проверки.".to_string())?;
if self.session.get(view.id).is_some_and(|doc| !doc.allows_copying()) { return Err("Этот PDF запрещает извлечение текста.".into()); }
let text = view.selected_text().filter(|t| !t.is_empty()).ok_or_else(|| "Выделите текст в PDF, затем нажмите «Орфография · русский». Скан без распознанного текста сначала нужно распознать.".to_string())?;
texts.push("Выделенный текст PDF", &text);
''',
'lightcraft':r'''
for id in self.session.selected().into_iter().take(2000) {
    if let Some(photo) = self.session.catalog.photo(id) {
        texts.push("Название фотографии", &photo.meta.title);
        texts.push("Подпись фотографии", &photo.meta.caption);
        texts.push("Альтернативный текст", &photo.meta.alt_text);
        texts.push("Описание фотографии", &photo.meta.extended_description);
    }
}
''',
'filmcraft':r'''
for item in self.session.project.items.values().take(10000) {
    for (label, text) in &item.metadata { texts.push(format!("{}: {}", item.name, label), text); }
    if let Some(sequence) = item.as_sequence() {
        for track in &sequence.caption_tracks { for caption in &track.captions { texts.push(format!("{} / {}", item.name, track.name), &caption.plain_lines().join("\n")); } }
        for track in sequence.all_tracks() { for clip in &track.items { for layer in clip.graphic_layers() {
            if let Some(param) = layer.params.get("text") {
                if let Ok(value) = serde_json::to_value(&param.value) { crate::proof_panel::strings(&mut texts, "Титры", &value, 0); }
            }
        } } }
    }
}
''',
'effectcraft':r'''
if let Some(comp) = self.session.active_comp() {
    for layer in comp.layers.iter().take(2000) {
        if let Some(doc) = effectcraft_engine::commands::text_edit::layer_doc(&self.session, layer.id) { texts.push(layer.name.clone(), &doc.text); }
    }
}
''',
'soundcraft':r'''
let session = self.engine.session();
texts.push("Название проекта", &session.name);
texts.push("Комментарий проекта", &session.comments);
for track in session.tracks.iter().take(2000) { texts.push("Название дорожки", &track.name); texts.push(format!("Комментарий: {}", track.name), &track.comments); }
for marker in session.markers.iter().take(2000) { texts.push("Название маркера", &marker.name); texts.push(format!("Комментарий: {}", marker.name), &marker.comments); }
''',
}

def main():
    for app,reader in READERS.items():
        module(app,'ui-egui')
        src=ROOT/'tmp/artcraft'/app/'repo/crates/ui-egui/src'
        manifest=src.parent/'Cargo.toml'
        content=manifest.read_text('utf-8')
        if (ROOT/'scripts/artcraft-android/platform/Cargo.toml').exists() and 'artcraft-android-platform' not in content:
            content+='\n[target.\'cfg(target_os = "android")\'.dependencies]\nartcraft-android-platform = { path = "../../../../../../scripts/artcraft-android/platform" }\n'
            temporary=manifest.parent.parent/'spelling-manifest.tmp'
            temporary.write_text(content,encoding='utf-8')
            temporary.replace(manifest)
        (src/'proof_panel.rs').write_text((HERE/'proof_panel.rs').read_text('utf-8'),encoding='utf-8')
        lib=src/'lib.rs'
        if 'mod proof_panel;' not in lib.read_text('utf-8'):
            with lib.open('a',encoding='utf-8') as f:f.write('\nmod proof_panel;\n')
        file='lib.rs' if app=='cadcraft' else 'app.rs'
        signature='pub fn ui(&mut self, ui: &mut egui::Ui) {'
        if app=='pdfcraft':signature='fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {'
        elif app in ['filmcraft','effectcraft']:signature='fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {'
        reader=re.sub(r'for (\w+) in ([^\n]+)\.take\((\d+)\) \{',lambda m:f'for (proof_index, {m[1]}) in {m[2]}.enumerate() {{\n    if proof_index >= {m[3]} {{ texts.truncated = true; break; }}',reader)
        injection='\n        crate::proof_panel::show(ui, || {\n            let mut texts = crate::proof_panel::Texts::new("'+SCOPES[app]+'");\n'+reader+'\n            Ok(texts)\n        });'
        path=f'crates/ui-egui/src/{file}'
        current=(src/file).read_text('utf-8')
        if 'crate::proof_panel::show(ui,' not in current:edit(app,path,[(signature,signature+injection)])
        else:
            start=current.index('\n        crate::proof_panel::show(ui,')
            end=current.index('\n        });',start)+len('\n        });')
            (src/file).write_text(current[:start]+injection+current[end:],encoding='utf-8')
        print('Prepared Russian review panel:',app)

if __name__=='__main__':main()
