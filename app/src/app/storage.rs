use std::io::SeekFrom;
use std::path::Path;
use std::sync::Arc;
use dashmap::DashMap;
use eframe::Storage;
use ron::ser::PrettyConfig;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use crate::app::popup::{handle_display_popup_arc, ArcPopupStore};
use crate::get_runtime;

#[derive(Default)]
pub struct FileStore {
    ron_filepath: Option<Arc<Path>>,
    kv: Arc<DashMap<String, String>>,
    dirty: bool,
    last_save_join_handle: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
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
    pub async fn from_ron_filepath(ron_filepath: Arc<Path>) -> (Option<std::io::Error>, Self) {
        log::debug!("Loading app state from {:?}…", ron_filepath);
        let okv = match read_ron(&ron_filepath).await{
            Err(err) => (Some(err), Arc::default()),
            Ok(kv) => (None, kv),
        };
        (okv.0, Self {
            kv: okv.1,
            ron_filepath: Some(ron_filepath.clone()),
            dirty: false,
            last_save_join_handle: None,
        })
    }

    pub const fn get_file_path(&self) -> Option<&Arc<Path>>{
        self.ron_filepath.as_ref()
    }

    pub async fn change_ron_file(&mut self, ron_filepath: Arc<Path>) -> std::io::Result<()> {
        save_to_disk(self.ron_filepath.clone(), self.kv.clone()).await?;
        self.ron_filepath = Some(ron_filepath);
        Ok(())
    }

    fn _set_string(&mut self, key: &str, value: String){
        self.kv.insert(key.to_owned(), value);
        self.dirty = true;
    }
    pub(super) fn flush(&mut self, popups: Option<ArcPopupStore>) {
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
                        Ok(Ok(())) => {}
                        Ok(Err(err)) => {
                            log::warn!("Error whilst saving to disk: {err}");
                            if let Some(popups) = popups {
                                handle_display_popup_arc(
                                    &popups,
                                    "There was an error saving this project.",
                                    &err,
                                    "Error Saving Project"
                                )
                            }
                        }
                        Err(err) => {
                            log::warn!("Error whilst saving to disk: {err}");
                            if let Some(popups) = popups {
                                handle_display_popup_arc(
                                    &popups,
                                    "There was a severe error saving this project. \nSomething definitely went majorly wrong.",
                                    &err,
                                    "Error Saving Project"
                                )
                            }
                        }
                    }
                }
                save_to_disk(ron_filepath, kv).await
            }));
        }
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
        self.flush(None)
    }
}

async fn save_to_disk(file_path: Option<Arc<Path>>, kv: Arc<DashMap<String, String>>) -> std::io::Result<()> {
    let file_path = match file_path {
        None => return Err(std::io::Error::from(std::io::ErrorKind::NotFound)),
        Some(file_path) => file_path,
    };
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
                        return Err(err);
                    }
                    if let Err(err) = file.write_all(out.as_bytes()).await{
                        log::warn!("Failed to write file contents: {}", err);
                        return Err(err);
                    }
                    if let Err(err) = file.flush().await{
                        log::warn!("Failed to flush file contents: {}", err);
                        return Err(err);
                    }

                    log::trace!("Persisted to {:?}", file_path);
                    Ok(())
                },
                Err(err) => {
                    log::warn!("Failed to serialize app state: {}", err);
                    Err(std::io::Error::other(err))
                },
            }
        }
        Err(err) => {
            log::warn!("Failed to create file {file_path:?}: {err}");
            Err(err)
        }
    }
}

async fn read_ron<T>(ron_path: &Arc<Path>) -> std::io::Result<T>
    where
        T: serde::de::DeserializeOwned,
{
    ron::de::from_str(tokio::fs::read_to_string(ron_path).await?.as_str())
        .map_err(std::io::Error::other)
}
