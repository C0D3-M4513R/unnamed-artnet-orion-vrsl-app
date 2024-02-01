#![allow(dead_code)] //todo: re-check, once the major implementation of this app has been done.
use std::io::SeekFrom;
use std::path::Path;
use std::sync::Arc;
use dashmap::DashMap;
use eframe::{Storage, storage_dir};
use ron::ser::PrettyConfig;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use crate::get_runtime;

pub struct FileStore {
    ron_filepath: Arc<Path>,
    kv: Arc<DashMap<String, String>>,
    dirty: bool,
    last_save_join_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Drop for FileStore {
    fn drop(&mut self) {
        if let Some(join_handle) = self.last_save_join_handle.take() {
            if let Err(err) = get_runtime().block_on(join_handle){
                log::warn!("Error whilst saving to disk: {}", err);
            }
        }
    }
}

impl FileStore {
    /// Store the state in this .ron file.
    ///
    /// The user of this function is responsible for making sure,
    /// that the `ron_filepath` can be read from.
    /// If the `ron_filepath` cannot be read from,
    /// this storage will just take the default value
    fn from_ron_filepath(ron_filepath: Arc<Path>) -> Option<Self> {
        log::debug!("Loading app state from {:?}…", ron_filepath);
        Some(Self {
            kv: read_ron(&ron_filepath)?,
            ron_filepath,
            dirty: false,
            last_save_join_handle: None,
        })
    }

    /// Find a good place to put the files that the OS likes.
    pub fn from_app_id(app_id: &str) -> Option<Self> {
        storage_dir(app_id).map_or_else(||{
            log::warn!("Saving disabled: Failed to find path to data_dir.");
            None
        }, |data_dir|{
            if let Err(err) = std::fs::create_dir_all(&data_dir) {
                log::warn!(
                    "Saving disabled: Failed to create app path at {:?}: {}",
                    data_dir,
                    err
                );
                None
            } else {
                Self::from_ron_filepath(Arc::from(data_dir.join("app.ron")))
            }
        })
    }
    fn _set_string(&mut self, key: &str, value: String){
        self.kv.insert(key.to_owned(), value);
        self.dirty = true;
    }
}

impl Storage for FileStore {
    fn get_string(&self, key: &str) -> Option<String> {
        self.kv.get(key).map(|x|x.key().clone())
    }

    fn set_string(&mut self, key: &str, value: String) {
        let kvo = self.kv.get(key);
        let needs_set = match kvo {
            None => true,
            Some(ref kv) if kv.key() != key => true,
            _ => false
        };
        drop(kvo);
        if needs_set {self._set_string(key, value)}
    }

    fn flush(&mut self) {
        if self.dirty {
            self.dirty = false;

            let last_save_join_handle = self.last_save_join_handle.take();
            let ron_filepath = self.ron_filepath.clone();
            let kv = self.kv.clone();
            self.last_save_join_handle = Some(tokio::spawn(async move {
                match last_save_join_handle{
                    None => {}
                    // wait for previous save to complete.
                    Some(v) => match v.await{
                        Ok(()) => {}
                        Err(e) => {
                            log::warn!("Error whilst saving to disk: {}", e);
                        }
                    }
                }
                save_to_disk(ron_filepath, kv).await
            }));
        }
    }
}

async fn save_to_disk(file_path: Arc<Path>, kv: Arc<DashMap<String, String>>) {

    if let Some(parent_dir) = file_path.parent() {
        if !parent_dir.exists() {
            if let Err(err) = tokio::fs::create_dir_all(parent_dir).await {
                log::warn!("Failed to create directory {parent_dir:?}: {err}");
            }
        }
    }

    match tokio::fs::File::create(&file_path).await {
        Ok(mut file) => {
            let config = PrettyConfig::default();
            match ron::ser::to_string_pretty(&kv, config){
                Ok(out) => {
                    if let Err(err) = file.seek(SeekFrom::Start(0)).await {
                        log::warn!("Failed to seek to start of file: {}", err);
                        return;
                    }
                    if let Err(err) = file.write_all(out.as_bytes()).await{
                        log::warn!("Failed to write file contents: {}", err);
                        return;
                    }
                    if let Err(err) = file.flush().await{
                        log::warn!("Failed to flush file contents: {}", err);
                        return;
                    }

                    log::trace!("Persisted to {:?}", file_path)
                },
                Err(err) => log::warn!("Failed to serialize app state: {}", err),
            };
        }
        Err(err) => {
            log::warn!("Failed to create file {file_path:?}: {err}");
        }
    }
}

fn read_ron<T>(ron_path: impl AsRef<Path>) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
{
    match std::fs::File::open(ron_path) {
        Ok(file) => {
            let reader = std::io::BufReader::new(file);
            match ron::de::from_reader(reader) {
                Ok(value) => Some(value),
                Err(err) => {
                    log::warn!("Failed to parse RON: {}", err);
                    None
                }
            }
        }
        Err(_err) => {
            // File probably doesn't exist. That's fine.
            None
        }
    }
}
