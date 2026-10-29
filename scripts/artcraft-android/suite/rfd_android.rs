//! Android file requests, rendered by the native egui host. No desktop subprocesses.
use crate::backend::*;
use crate::message_dialog::MessageDialogResult;
use crate::{FileDialog, FileHandle, MessageDialog};
use std::{
    collections::VecDeque,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex, OnceLock},
    task::{Context, Poll, Waker},
};

#[derive(Clone, Copy)]
pub enum Kind {
    Open,
    Save,
    Folder,
}
#[derive(Default)]
struct State {
    result: Option<Option<Vec<PathBuf>>>,
    waker: Option<Waker>,
}
pub struct Request {
    pub kind: Kind,
    pub name: String,
    state: Arc<Mutex<State>>,
}
impl Request {
    pub fn finish(&self, paths: Option<Vec<PathBuf>>) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.result = Some(paths);
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}
impl Drop for Request {
    fn drop(&mut self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.result.is_none() {
            state.result = Some(None);
        }
        if let Some(waker) = state.waker.take() {
            waker.wake();
        }
    }
}
fn queue() -> &'static Mutex<VecDeque<Request>> {
    static QUEUE: OnceLock<Mutex<VecDeque<Request>>> = OnceLock::new();
    QUEUE.get_or_init(Mutex::default)
}
pub fn take_request() -> Option<Request> {
    queue()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .pop_front()
}
struct Reply(Arc<Mutex<State>>);
impl Future for Reply {
    type Output = Option<Vec<PathBuf>>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
fn request(dialog: FileDialog, kind: Kind) -> Reply {
    let state = Arc::new(Mutex::new(State::default()));
    queue()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .push_back(Request {
            kind,
            name: dialog.file_name.unwrap_or_default(),
            state: state.clone(),
        });
    Reply(state)
}
fn one(dialog: FileDialog, kind: Kind) -> DialogFutureType<Option<FileHandle>> {
    let reply = request(dialog, kind);
    Box::pin(async move {
        reply
            .await
            .and_then(|paths| paths.into_iter().next())
            .map(FileHandle::from)
    })
}
fn many(dialog: FileDialog, kind: Kind) -> DialogFutureType<Option<Vec<FileHandle>>> {
    let reply = request(dialog, kind);
    Box::pin(async move {
        reply
            .await
            .map(|paths| paths.into_iter().map(FileHandle::from).collect())
    })
}
impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        one(self, Kind::Open)
    }
    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        many(self, Kind::Open)
    }
}
impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        one(self, Kind::Folder)
    }
    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        many(self, Kind::Folder)
    }
}
impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        one(self, Kind::Save)
    }
}
// PdfCraft uses asynchronous pickers exclusively. Synchronous desktop APIs cannot
// wait on their own render thread; report cancellation instead of deadlocking it.
impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        None
    }
    fn pick_files(self) -> Option<Vec<PathBuf>> {
        None
    }
}
impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        None
    }
    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        None
    }
}
impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        None
    }
}
impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        MessageDialogResult::Cancel
    }
}
impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        Box::pin(async { MessageDialogResult::Cancel })
    }
}
