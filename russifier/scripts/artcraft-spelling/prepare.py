"""Apply local Russian proofing support to the existing ArtCraft source checkouts."""
from pathlib import Path
import hashlib, json, re

ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
DICT=ROOT/'scripts/artcraft-russian/dictionaries'
BACKUP=ROOT/'tmp/artcraft/backups/before-russian-spelling'

def edit(app, path, changes):
    target=ROOT/'tmp/artcraft'/app/'repo'/path
    text=target.read_text('utf-8')
    old=text
    for before,after in changes:
        if after in text or re.sub(r'\s+', '', after) in re.sub(r'\s+', '', text):
            continue
        if before in text:
            text=text.replace(before,after,1)
        elif after not in text:
            raise RuntimeError(f'Patch context missing: {app}/{path}: {before[:80]}')
    if text!=old:
        backup=BACKUP/app/path
        backup.parent.mkdir(parents=True,exist_ok=True)
        if not backup.exists():backup.write_text(old,encoding='utf-8')
        target.write_text(text,encoding='utf-8')

def embedded(name,kind='bytes'):
    return f'include_{kind}!("{(DICT/name).as_posix()}")'

def module(app,crate):
    target=ROOT/'tmp/artcraft'/app/'repo/crates'/crate/'src/russian_spell.rs'
    code=(HERE/'russian_spell.rs').read_text('utf-8')
    for name in ['ru_RU.aff','ru_RU.dic']:
        code=code.replace(f'include_bytes!("../artcraft-russian/dictionaries/{name}")',embedded(name))
    target.write_text(code,encoding='utf-8')
    lib=target.with_name('lib.rs')
    text=lib.read_text('utf-8').replace('\nmod russian_spell;\n','\n')
    lib.write_text(text.rstrip()+'\n\nmod russian_spell;\n',encoding='utf-8')

def main():
    for name,info in json.loads((DICT/'sources.json').read_text('utf-8')).items():
        assert hashlib.sha256((DICT/name).read_bytes()).hexdigest()==info['sha256'],name
    # WordCraft retains its richer Russian grammar/morphology engine and user override.
    fallback=f'''return AffixDict::from_hunspell({embedded('ru_RU.aff','str')}, {embedded('ru_RU.dic','str')}, &["Russian dictionary: Alexander I. Lebedev; see ArtCraft spelling notice"])
            .unwrap_or_else(|error| {{ log::error!("Russian dictionary: {{error}}"); AffixDict::default() }});'''
    edit('wordcraft','crates/proof/src/ru/mod.rs',[
        ('log::info!("Russian spelling: no Hunspell ru_RU dictionary installed");\n        return AffixDict::default();',fallback)])
    edit('wordcraft','crates/engine/src/prefs.rs',[('default_language: None,','default_language: Some("ru-RU".into()),')])
    edit('wordcraft','crates/ui-egui/src/app/prefs.rs',[
        ('self.session.prefs = std::mem::take(&mut self.ui.editing);','self.session.prefs = std::mem::take(&mut self.ui.editing);\n        if self.session.prefs.default_language.is_none() { self.session.prefs.default_language = Some("ru-RU".into()); }')])
    # DeckCraft already has Hunspell morphology, including per-run languages.
    edit('deckcraft','crates/engine/src/spell.rs',[
        ('    if let Ok(mut c) = cache.lock() {\n        c.insert(key, loaded.clone());',f'''    let loaded = loaded.or_else(|| lang.replace('_', "-").to_ascii_lowercase().starts_with("ru").then(|| Arc::new(Dictionary::parse({embedded('ru_RU.aff')}, {embedded('ru_RU.dic')}))));
    if let Ok(mut c) = cache.lock() {{
        c.insert(key, loaded.clone());'''),
        ('lang: lang_of(c.1), repeated: None','lang: if part.chars().any(|c| matches!(c, \'А\'..=\'я\' | \'Ё\' | \'ё\')) { "ru-RU".into() } else { lang_of(c.1) }, repeated: None')])
    for app,crate in [('gridcraft','engine'),('designcraft','engine'),('photocraft','text')]:module(app,crate)
    edit('gridcraft','crates/engine/src/cmd/spelling.rs',[
        ('    let dict = dictionary();\n    if dict.contains(&l)', '    if crate::russian_spell::is_russian(w) { return s.prefs.user_dictionary.iter().any(|u| u.to_lowercase() == l) || crate::russian_spell::russian().check(w); }\n    let dict = dictionary();\n    if dict.contains(&l)'),
        ('fn suggestions(w: &str) -> Vec<String> {','fn suggestions(w: &str) -> Vec<String> {\n    if crate::russian_spell::is_russian(w) { return crate::russian_spell::russian().suggest(w, 6); }'),
        ('    if dictionary().is_empty() {\n        return Err(EngineError::Other("No dictionary is available on this system (GridCraft uses the system word list).".into()));\n    }','    // Russian is bundled even when the OS has no English word list.'),
        ("w.chars().count() < 2 || w.chars().all(char::is_uppercase) || !w.is_ascii()", "w.chars().count() < 2 || w.chars().all(char::is_uppercase) || (!w.is_ascii() && !crate::russian_spell::is_russian(w)) || (w.is_ascii() && dictionary().is_empty())")])
    edit('designcraft','crates/engine/src/cmd/spelling.rs',[
        ('pub fn known(dict: &Dictionary, user: &[String], w: &str) -> bool {','pub fn known(dict: &Dictionary, user: &[String], w: &str) -> bool {\n    if crate::russian_spell::is_russian(w) { return user.iter().any(|u| u.to_lowercase() == w.to_lowercase()) || crate::russian_spell::russian().check(w); }'),
        ('pub fn suggest(dict: &Dictionary, w: &str) -> Vec<String> {','pub fn suggest(dict: &Dictionary, w: &str) -> Vec<String> {\n    if crate::russian_spell::is_russian(w) { return crate::russian_spell::russian().suggest(w, 6); }'),
        ('!foreign.iter().any(|r| r.contains(&start))','(crate::russian_spell::is_russian(word) || !foreign.iter().any(|r| r.contains(&start)))')])
    edit('photocraft','crates/text/src/spell.rs',[
        ('        let lower = w.to_lowercase();\n        if user.contains(&lower)', '        let lower = w.to_lowercase();\n        if crate::russian_spell::is_russian(w) { return user.contains(&lower) || crate::russian_spell::russian().check(w); }\n        if user.contains(&lower)'),
        ('    pub fn suggest(&self, word: &str, max: usize) -> Vec<String> {','    pub fn suggest(&self, word: &str, max: usize) -> Vec<String> {\n        if crate::russian_spell::is_russian(word) { return crate::russian_spell::russian().suggest(word, max); }')])
    print('Prepared Russian proofing in five existing engines')

if __name__=='__main__':main()
