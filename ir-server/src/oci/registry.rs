use async_trait::async_trait;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use tokio::fs;
use tokio_util::io;

use super::application::Application;
use super::digest::Digest;
use crate::error::RegistryError;
use crate::registry::ImageRegistry;
use crate::RegistryResult;

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistry(format!($($arg)+))))
}

#[derive(Debug, Default)]
pub struct Registry
{
    apps: HashMap<String, Application>,
}

impl Registry
{
    pub fn import(path: &str) -> RegistryResult<Self>
    {
        info!("Loading registry from: \"{}\"", path);

        let mut reg = Registry::default();

        let reg_path = Path::new(path);
        if !reg_path.is_dir() {
            err!("Registry path \"{}\" is not a directory", path)?;
        }

        for file in std::fs::read_dir(reg_path)? {
            let app_path = file?.path();
            if app_path.is_dir() {
                let app_name = app_path
                    .file_name()
                    .unwrap_or(OsStr::new(""))
                    .to_string_lossy();
                debug!("Application \"{}\" found", app_name);
                match Application::import(&app_path) {
                    Ok(a) => {
                        reg.apps.insert(app_name.to_string(), a);
                    }
                    Err(e) => error!("Failed to load app \"{}\": {}", app_name, e),
                };
            } else {
                warn!(
                    "Non directory \"{}\" found in registry dir, ignoring",
                    app_path.display()
                );
            }
        }

        Ok(reg)
    }
}

#[async_trait]
impl ImageRegistry for Registry
{
    fn get_tags(&self, app: &str) -> Option<Vec<String>>
    {
        let app = self.apps.get(app)?;

        let tags = app.get_tags();
        Some(tags.keys().map(|k| k.to_string()).collect())
    }

    async fn get_manifest(&self, app: &str, reference: &str) -> Option<io::ReaderStream<fs::File>>
    {
        let app = self.apps.get(app)?;

        // assume that reference is a digest first
        let content = match Digest::try_from(reference) {
            Ok(digest) => app.get_manifests().get(&digest)?,
            Err(_) => app.get_tags().get(reference)?,
        };

        let file = match fs::File::open(&content.path).await {
            Ok(file) => file,
            Err(err) => {
                error!("Error opening \"{}\": {}", content.path.display(), err);
                return None;
            }
        };

        Some(tokio_util::io::ReaderStream::new(file))
    }

    async fn get_blob(&self, app: &str, digest: &str) -> Option<io::ReaderStream<fs::File>>
    {
        let a = self.apps.get(app)?;

        let content = match Digest::try_from(digest) {
            Ok(digest) => a.get_blobs().get(&digest)?,
            Err(_) => return None,
        };

        let file = match fs::File::open(&content.path).await {
            Ok(file) => file,
            Err(err) => {
                error!("Error opening \"{}\": {}", content.path.display(), err);
                return None;
            }
        };

        Some(tokio_util::io::ReaderStream::new(file))
    }
}
