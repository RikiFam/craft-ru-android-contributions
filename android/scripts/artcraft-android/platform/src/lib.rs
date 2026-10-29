//! Shared Android SDK bridge. Picker operations never block the native event loop.
use jni::{
    JavaVM,
    objects::{GlobalRef, JObject, JString, JValue},
};
use std::sync::{OnceLock, mpsc};

struct Platform {
    vm: JavaVM,
    activity: GlobalRef,
}
static PLATFORM: OnceLock<Platform> = OnceLock::new();
#[derive(Clone)]
struct Command {
    id: String,
    params: serde_json::Value,
}
struct Deferred {
    pick: Pick,
    command: Command,
    direct: bool,
}
thread_local! {
    static ENTRY: std::cell::RefCell<Option<Command>> = const { std::cell::RefCell::new(None) };
    static DEFERRED: std::cell::RefCell<Option<Deferred>> = const { std::cell::RefCell::new(None) };
    static ANSWER: std::cell::RefCell<Option<Vec<String>>> = const { std::cell::RefCell::new(None) };
}
/// Retains the outer command so old synchronous picker services can resume it.
pub fn command<T>(id: &str, params: &serde_json::Value, run: impl FnOnce() -> T) -> T {
    let outer = ENTRY.with(|entry| {
        let mut entry = entry.borrow_mut();
        if entry.is_some() {
            false
        } else {
            *entry = Some(Command {
                id: id.into(),
                params: params.clone(),
            });
            true
        }
    });
    struct Reset(bool);
    impl Drop for Reset {
        fn drop(&mut self) {
            if self.0 {
                ENTRY.with(|entry| entry.borrow_mut().take());
            }
        }
    }
    let _reset = Reset(outer);
    run()
}
pub fn deferred_pick(kind: &str, name: &str, fallback: &str) -> Option<String> {
    if let Some(answer) = ANSWER.with(|answer| answer.borrow_mut().take()) {
        return answer.into_iter().next();
    }
    if DEFERRED.with(|pending| pending.borrow().is_some()) {
        return None;
    }
    let entry = ENTRY.with(|entry| entry.borrow().clone());
    let direct = entry.is_none();
    let command = entry.unwrap_or(Command {
        id: fallback.into(),
        params: serde_json::json!({}),
    });
    match Pick::start(kind, name) {
        Ok(pick) => DEFERRED.with(|pending| {
            *pending.borrow_mut() = Some(Deferred {
                pick,
                command,
                direct,
            })
        }),
        Err(error) => report(error),
    }
    None
}
pub fn replay() -> Option<(String, serde_json::Value)> {
    DEFERRED.with(|pending| {
        let result = pending.borrow().as_ref()?.pick.poll();
        match result {
            Ok(None) => None,
            Ok(Some(paths)) => {
                let mut request = pending.borrow_mut().take()?;
                if paths.is_empty() {
                    return None;
                }
                if request.direct {
                    request.command.params["path"] = paths[0].clone().into();
                } else {
                    ANSWER.with(|answer| *answer.borrow_mut() = Some(paths));
                }
                Some((request.command.id, request.command.params))
            }
            Err(error) => {
                pending.borrow_mut().take();
                report(error);
                None
            }
        }
    })
}
pub fn finish_replay() {
    ANSWER.with(|answer| answer.borrow_mut().take());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_commands_keep_the_original_request() {
        let original = serde_json::json!({"format": "pdf", "pages": [1, 3]});
        command("file.export", &original, || {
            command("internal.write", &serde_json::Value::Null, || {
                ENTRY.with(|entry| {
                    let entry = entry.borrow();
                    let captured = entry.as_ref().unwrap();
                    assert_eq!(captured.id, "file.export");
                    assert_eq!(captured.params, original);
                });
            });
        });
        ENTRY.with(|entry| assert!(entry.borrow().is_none()));
    }

    #[test]
    fn command_scope_clears_after_a_failed_operation() {
        let result: Result<(), &str> =
            command("save", &serde_json::Value::Null, || Err("read only"));
        assert_eq!(result, Err("read only"));
        ENTRY.with(|entry| assert!(entry.borrow().is_none()));
    }

    #[test]
    fn replay_answer_is_consumed_once() {
        ANSWER.with(|answer| *answer.borrow_mut() = Some(vec!["/private/file.pdf".into()]));
        assert_eq!(
            deferred_pick("save", "ignored", "ignored"),
            Some("/private/file.pdf".into())
        );
        ANSWER.with(|answer| assert!(answer.borrow().is_none()));
        finish_replay();
    }
}
static ERRORS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());
pub fn report(error: String) {
    log::error!("{error}");
    ERRORS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push(error);
}
pub fn take_errors() -> Vec<String> {
    std::mem::take(
        &mut *ERRORS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner),
    )
}
pub fn pick_paths(kind: &str, name: &str) -> mpsc::Receiver<Vec<String>> {
    let receiver = pick_async(kind, name);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let paths = match receiver.recv() {
            Ok(Ok(paths)) => paths,
            Ok(Err(error)) => {
                report(error);
                Vec::new()
            }
            Err(error) => {
                report(error.to_string());
                Vec::new()
            }
        };
        let _ = tx.send(paths);
    });
    rx
}
pub fn pick_path_future(
    kind: &str,
    name: &str,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<std::path::PathBuf>>>> {
    let receiver = pick_paths(kind, name);
    Box::pin(std::future::poll_fn(move |cx| match receiver.try_recv() {
        Ok(paths) => std::task::Poll::Ready(paths.into_iter().next().map(std::path::PathBuf::from)),
        Err(mpsc::TryRecvError::Disconnected) => std::task::Poll::Ready(None),
        Err(mpsc::TryRecvError::Empty) => {
            cx.waker().wake_by_ref();
            std::task::Poll::Pending
        }
    }))
}

/// # Safety
/// Both pointers must belong to the current live AndroidApp. A global activity
/// reference is retained, and JNI attachment is scoped separately on each thread.
pub unsafe fn initialize(
    vm: *mut std::ffi::c_void,
    activity: *mut std::ffi::c_void,
) -> Result<(), String> {
    let vm = unsafe { JavaVM::from_raw(vm.cast()) }.map_err(|e| e.to_string())?;
    let reference = {
        let env = vm.attach_current_thread().map_err(|e| e.to_string())?;
        let borrowed = unsafe { JObject::from_raw(activity.cast()) };
        env.new_global_ref(&borrowed).map_err(|e| e.to_string())?
    };
    PLATFORM
        .set(Platform {
            vm,
            activity: reference,
        })
        .map_err(|_| "Android platform already initialized".into())
}
fn call<T>(
    f: impl FnOnce(&mut jni::JNIEnv, &JObject) -> jni::errors::Result<T>,
) -> Result<T, String> {
    let platform = PLATFORM.get().ok_or("Службы Android не подключены")?;
    let mut env = platform
        .vm
        .attach_current_thread()
        .map_err(|e| e.to_string())?;
    let result = env.with_local_frame(32, |env| f(env, platform.activity.as_obj()));
    if result.is_err() {
        let _ = env.exception_clear();
    }
    result.map_err(|e| format!("Ошибка службы Android: {e}"))
}
pub struct Pick {
    id: i32,
}
/// Unicode commits which NativeActivity cannot represent as AKeyEvent key codes.
pub fn committed_text() -> Result<Vec<String>, String> {
    let value = call(|env, activity| {
        let value = env
            .call_method(activity, "pollCommittedText", "()Ljava/lang/String;", &[])?
            .l()?;
        Ok(String::from(env.get_string(&JString::from(value))?))
    })?;
    serde_json::from_str(&value).map_err(|e| e.to_string())
}
impl Pick {
    pub fn start(kind: &str, name: &str) -> Result<Self, String> {
        call(|env, activity| {
            let kind = env.new_string(kind)?;
            let name = env.new_string(name)?;
            let id = env
                .call_method(
                    activity,
                    "select",
                    "(Ljava/lang/String;Ljava/lang/String;)I",
                    &[JValue::Object(&kind), JValue::Object(&name)],
                )?
                .i()?;
            Ok(Self { id })
        })
    }
    pub fn poll(&self) -> Result<Option<Vec<String>>, String> {
        let text = call(|env, activity| {
            let value = env
                .call_method(
                    activity,
                    "poll",
                    "(I)Ljava/lang/String;",
                    &[JValue::Int(self.id)],
                )?
                .l()?;
            if value.is_null() {
                return Ok(None);
            }
            Ok(Some(String::from(env.get_string(&JString::from(value))?)))
        })?;
        let Some(text) = text else { return Ok(None) };
        let value: serde_json::Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if let Some(error) = value.get("error").and_then(|v| v.as_str()) {
            return Err(error.into());
        }
        let paths = value
            .get("paths")
            .and_then(|v| v.as_array())
            .ok_or("Некорректный ответ выбора файлов")?;
        Ok(Some(
            paths
                .iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect(),
        ))
    }
}
pub fn pick_async(kind: &str, name: &str) -> mpsc::Receiver<Result<Vec<String>, String>> {
    let (tx, rx) = mpsc::channel();
    let request = Pick::start(kind, name);
    std::thread::spawn(move || {
        let answer = request.and_then(|request| {
            loop {
                match request.poll() {
                    Ok(Some(paths)) => break Ok(paths),
                    Err(e) => break Err(e),
                    _ => std::thread::sleep(std::time::Duration::from_millis(40)),
                }
            }
        });
        let _ = tx.send(answer);
    });
    rx
}
pub fn commit(path: &std::path::Path) -> Result<(), String> {
    let error = call(|env, activity| {
        let path = env.new_string(path.to_string_lossy())?;
        let answer = env
            .call_method(
                activity,
                "commitDocument",
                "(Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(&path)],
            )?
            .l()?;
        Ok(String::from(env.get_string(&JString::from(answer))?))
    })?;
    if error.is_empty() { Ok(()) } else { Err(error) }
}
pub fn print_pdf(bytes: &[u8], title: &str) -> Result<(), String> {
    let directory = std::path::PathBuf::from(call(|env, activity| {
        let value = env
            .call_method(activity, "printCache", "()Ljava/lang/String;", &[])?
            .l()?;
        Ok(String::from(env.get_string(&JString::from(value))?))
    })?);
    let file = directory.join(format!(
        "artcraft-print-{}.pdf",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos()
    ));
    std::fs::write(&file, bytes).map_err(|e| e.to_string())?;
    let error = call(|env, activity| {
        let path = env.new_string(file.to_string_lossy())?;
        let title = env.new_string(title)?;
        let answer = env
            .call_method(
                activity,
                "printDocument",
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
                &[JValue::Object(&path), JValue::Object(&title)],
            )?
            .l()?;
        Ok(String::from(env.get_string(&JString::from(answer))?))
    })?;
    if error.is_empty() { Ok(()) } else { Err(error) }
}
