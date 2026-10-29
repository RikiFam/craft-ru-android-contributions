use photocraft_format::RecoveryStore;
use photocraft_ui_egui::{Recoverable, Services};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex, PoisonError},
};

pub fn services(directory: PathBuf) -> Services {
    let store = Arc::new(Mutex::new(RecoveryStore::new(directory.clone())));
    let save = store.clone();
    let results = store.clone();
    let flush = store.clone();
    let discard = store.clone();
    Services {
        autosave: Some(Box::new(move |doc, revision, path| {
            save.lock()
                .unwrap_or_else(PoisonError::into_inner)
                .autosave_checked(doc, revision, path.map(str::to_owned))
        })),
        autosave_results: Some(Box::new(move || {
            results
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take_completed()
        })),
        flush_autosaves: Some(Box::new(move || {
            flush.lock().unwrap_or_else(PoisonError::into_inner).flush()
        })),
        discard_autosave: Some(Box::new(move |id| {
            if let Err(e) = discard
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .discard(id)
            {
                log::warn!("Recovery: {e}");
            }
        })),
        recover: Some(Box::new(move || {
            photocraft_format::list_recovery(&directory)
                .into_iter()
                .map(|entry| Recoverable {
                    key: entry.info.key.clone(),
                    name: entry.info.document_name.clone(),
                    path: entry.info.original_path.clone(),
                    load: Box::new(move || {
                        photocraft_format::recover(&entry).map_err(|e| e.to_string())
                    }),
                })
                .collect()
        })),
        adopt_autosave: Some(Box::new(move |id, key| {
            store
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .adopt(id, key);
        })),
        ..Default::default()
    }
}
