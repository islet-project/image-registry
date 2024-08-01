use async_trait::async_trait;
use ir_protocol::Manifest;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use tokio::fs;
use tokio_util::io;
use uuid::Uuid;

use super::application::Application;
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
    fn get_manifest(&self, _uuid: &Uuid) -> Option<&Manifest>
    {
        None
    }

    async fn get_image(&self, _uuid: &Uuid) -> Option<io::ReaderStream<fs::File>>
    {
        None
    }
}
