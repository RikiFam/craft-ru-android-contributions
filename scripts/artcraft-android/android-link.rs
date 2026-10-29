//! Keep NDK command-line paths ASCII on Windows without moving or rebuilding artifacts.
use std::{env, process::{Command, ExitCode}};

fn map_path(arg: &str, from: &str, to: &str) -> String {
    arg.replace(&from.replace('\\', "\\\\"), &to.replace('\\', "\\\\"))
        .replace(from, to).replace(&from.replace('\\', "/"), &to.replace('\\', "/"))
}

fn run() -> Result<i32, String> {
    let read = |key| env::var(key).map_err(|_| format!("Missing {key}"));
    let from = read("ARTCRAFT_LINK_FROM")?;
    let to = read("ARTCRAFT_LINK_TO")?;
    let clang = read("ARTCRAFT_NDK_CLANG")?;
    let args: Result<Vec<_>, String> = env::args_os().skip(1).map(|arg| {
        let arg = arg.to_str().ok_or("Invalid Unicode in linker argument")?;
        let mapped = if let Some(file) = arg.strip_prefix('@') {
            let text = std::fs::read_to_string(file).map_err(|e| format!("Response file: {e}"))?;
            let text = map_path(&text, &from, &to);
            if !text.is_ascii() { return Err("Unmapped Unicode in linker response file".into()); }
            let name = format!("{}.ascii.rsp", map_path(file, &from, &to));
            std::fs::write(&name, text).map_err(|e| e.to_string())?;
            format!("@{name}")
        } else { map_path(arg, &from, &to) };
        if !mapped.is_ascii() { return Err(format!("Unmapped non-ASCII linker path: {mapped}")); }
        Ok(mapped)
    }).collect();
    let status = Command::new(clang).args(args?).status().map_err(|e| e.to_string())?;
    Ok(status.code().unwrap_or(1))
}

fn main() -> ExitCode {
    match run() {
        Ok(0) => ExitCode::SUCCESS,
        Ok(_) => ExitCode::FAILURE,
        Err(error) => { eprintln!("Android linker: {error}"); ExitCode::FAILURE }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn maps_both_path_styles_without_touching_other_arguments() {
        let from = r"E:\Codex\ПК\tmp\android";
        let to = r"E:\Codex\AndroidLink";
        assert_eq!(super::map_path(&format!("-Wl,--version-script={from}\\list"), from, to), format!("-Wl,--version-script={to}\\list"));
        assert_eq!(super::map_path("E:/Codex/ПК/tmp/android/lib.so", from, to), "E:/Codex/AndroidLink/lib.so");
        assert_eq!(super::map_path("--target=aarch64-linux-android29", from, to), "--target=aarch64-linux-android29");
    }
}
