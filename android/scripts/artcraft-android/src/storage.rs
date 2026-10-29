use std::{
    io::Write,
    path::{Component, Path, PathBuf},
};

pub fn guard<T>(action: &str, operation: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or_else(|_| {
        Err(format!(
            "Не удалось {action}: внутренняя ошибка обработки. Документ остаётся открытым."
        ))
    })
}

pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or("Не удалось определить папку сохранения")?;
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    pending.write_all(bytes).map_err(|e| e.to_string())?;
    pending.as_file().sync_all().map_err(|e| e.to_string())?;
    pending.persist(path).map_err(|e| e.to_string())?;
    #[cfg(target_os = "android")]
    artcraft_android_platform::commit(path)?;
    Ok(())
}

pub fn save_path(directory: &Path, name: &str) -> Result<PathBuf, String> {
    let name = name.trim();
    let mut parts = Path::new(name).components();
    if !matches!(parts.next(), Some(Component::Normal(_)))
        || parts.next().is_some()
        || name.contains(['/', '\\'])
    {
        return Err("Введите имя файла без пути к папке".into());
    }
    Ok(directory.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn codec_panic_becomes_an_error_without_closing_the_editor() {
        let result: Result<(), String> = guard("открыть файл", || {
            panic!("synthetic broken decoder")
        });
        assert!(result.is_err());
        assert_eq!(guard("save", || Ok(42)).unwrap(), 42);
    }
    #[test]
    fn filename_cannot_escape_selected_folder() {
        for name in ["", "..", "../file.png", "a/b.png", "a\\b.png", "/file.png"] {
            assert!(save_path(Path::new("pictures"), name).is_err(), "{name}");
        }
        assert_eq!(
            save_path(Path::new("pictures"), "Кот.png").unwrap(),
            Path::new("pictures").join("Кот.png")
        );
    }
    #[test]
    fn atomic_save_replaces_complete_file_and_keeps_no_temporary_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Кот.png");
        std::fs::write(&path, b"old").unwrap();
        write_atomic(&path, b"complete new content").unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), b"complete new content");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}
