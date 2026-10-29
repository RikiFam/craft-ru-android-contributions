//! Russian interface for the local build. Only display text calls this module.
//! Command ids, user documents, file formats and undo operations retain their source values.
//! Set CRAFT_INTERFACE_LANGUAGE=en before launching to use the original English labels.
use std::collections::HashMap;
use std::sync::OnceLock;

fn catalog() -> &'static HashMap<String, String> {
    static CATALOG: OnceLock<HashMap<String, String>> = OnceLock::new();
    CATALOG.get_or_init(|| serde_json::from_str(include_str!("ru.json")).unwrap_or_default())
}

pub fn russian() -> bool {
    static RUSSIAN: OnceLock<bool> = OnceLock::new();
    *RUSSIAN.get_or_init(|| {
        std::env::var("CRAFT_INTERFACE_LANGUAGE")
            .map(|v| !v.eq_ignore_ascii_case("en"))
            .unwrap_or(!cfg!(test))
    })
}

/// A borrowed lookup: unknown labels are kept intact, never guessed or altered.
pub fn translate(source: &str) -> &str {
    catalog().get(source).or_else(|| catalog().get(source.trim())).map(String::as_str).unwrap_or(source)
}

pub fn t(source: &str) -> &str {
    if russian() { translate(source) } else { source }
}

/// Substitute named values once, preserving user names and paths verbatim.
pub fn message(source: &str, values: &[(&str, String)]) -> String {
    fill(t(source), values)
}

fn fill(template: &str, values: &[(&str, String)]) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        if let Some(end) = rest.find('}')
            && let Some((_, value)) = values.iter().find(|(name, _)| *name == &rest[1..end]) {
            out.push_str(value);
            rest = &rest[end + 1..];
            continue;
        }
        out.push('{');
        rest = &rest[1..];
    }
    out.push_str(rest);
    out
}

pub fn document_name(source: &str) -> &str {
    if source == "Untitled" { t(source) } else { source }
}

/// Translate the generated, numeric time-unit captions used by the toolbar only.
pub fn time_value(source: &str) -> String {
    if !russian() { return source.to_owned(); }
    for (ending, translated) in [(" note", " ноты"), (" sec", " с"), (" ms", " мс"), (" bars", " такт."), (" bar", " такт"), (" samples", " отсч.")] {
        if let Some(number) = source.strip_suffix(ending) {
            return format!("{number}{translated}");
        }
    }
    t(source).to_owned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn message_values_are_not_retranslated_or_interpolated() {
        assert_eq!(super::fill("Сохранить «{name}»? {unknown}", &[("name", "File {name} 日本語".into())]), "Сохранить «File {name} 日本語»? {unknown}");
    }
    #[test]
    fn russian_basics_and_unknown_text() {
        assert_eq!(super::translate("File"), "Файл");
        assert_eq!(super::translate("Save"), "Сохранить");
        assert_eq!(super::translate("Open…"), "Открыть…");
        assert_eq!(super::translate("User document 42"), "User document 42");
        assert_eq!(super::translate("track.new"), "track.new");
    }

    #[test]
    fn catalog_is_valid_utf8_without_replacement_characters() {
        let data = include_str!("ru.json");
        assert!(!data.contains('\u{fffd}'));
        assert!(serde_json::from_str::<std::collections::HashMap<String, String>>(data).is_ok());
    }
}
