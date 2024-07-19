mod digest;
mod sha2;
mod validate;

use async_trait::async_trait;
use digest::Digest;
use ir_protocol::Manifest;
use log::{debug, error, info, warn};
use oci_spec::image::{
    Descriptor, ImageIndex, ImageManifest, MediaType, OciLayout, ANNOTATION_REF_NAME,
};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio_util::io;
use uuid::Uuid;
use validate::Validate;

use crate::error::RegistryError;
use crate::registry::ImageRegistry;
use crate::RegistryResult;

const OCI_LAYOUT: &str = "oci-layout";
const INDEX_JSON: &str = "index.json";
const BLOBS_SUBDIR: &str = "blobs";

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistryError(format!($($arg)+))))
}

#[derive(Debug, Default)]
struct Application
{
    path: PathBuf,
    tags: HashMap<String, PathBuf>,
    manifests: HashMap<Digest, PathBuf>,
    blobs: HashMap<Digest, PathBuf>,
}

#[derive(Debug, Default)]
pub struct Registry
{
    apps: HashMap<String, Application>,
}

impl Application
{
    fn find_orphans(&self) -> RegistryResult<Vec<Digest>>
    {
        let mut orphans = Vec::new();

        for algo in ["sha256", "sha512"] {
            let path = self.path.join(BLOBS_SUBDIR).join(algo);
            if !path.exists() {
                continue;
            }

            for file in std::fs::read_dir(path)? {
                let file = file?.path();
                let file_name = file.file_name().unwrap_or(OsStr::new("")).to_string_lossy();
                let digest = Digest::new_unchecked(algo.to_string(), file_name.to_string());

                match (self.blobs.get(&digest), self.manifests.get(&digest)) {
                    (None, None) => orphans.push(digest),
                    _ => (),
                }
            }
        }

        Ok(orphans)
    }

    fn import_descriptor(&mut self, desc: &Descriptor, layout_index: bool) -> RegistryResult<()>
    {
        let digest = Digest::try_from(desc.digest())?;
        let path = self.path.join(BLOBS_SUBDIR).join(digest.to_path());

        if !path.exists() {
            err!("The requested digest doesn't exist: {}", digest)?;
        }

        let desc_size: u64 = desc.size().try_into().or(err!("File size negative"))?;
        let file_size = std::fs::metadata(&path)?.len();
        if file_size != desc_size {
            err!("Wrong file length: {}, expected: {}", file_size, desc_size)?;
        }

        // For layout index descriptors do two additional things:
        // - verify hashes of linked manifests
        // - load tags from annotations
        if layout_index {
            if !sha2::verify(&path, &digest)? {
                err!("SHA failed for \"{}\"", path.display())?;
            }

            if let Some(anns) = desc.annotations() {
                if let Some(tag) = anns.get(ANNOTATION_REF_NAME) {
                    self.tags.insert(tag.to_string(), path.clone());
                }
            }
        }

        let media_type = desc.media_type();
        match media_type {
            MediaType::ImageIndex => {
                self.import_index(&path, false)?;
                self.manifests.insert(digest, path);
            }
            MediaType::ImageManifest => {
                self.import_manifest(&path)?;
                self.manifests.insert(digest, path);
            }
            MediaType::ImageConfig
            | MediaType::ImageLayer
            | MediaType::ImageLayerGzip
            | MediaType::ImageLayerZstd => {
                self.blobs.insert(digest, path);
            }
            m => err!("Unsupported media type: {}", m)?,
        }

        Ok(())
    }

    fn import_index(&mut self, path: &Path, layout_index: bool) -> RegistryResult<()>
    {
        let index = match ImageIndex::from_file(path) {
            Ok(i) => i,
            Err(e) => err!("Error importing \"{}\": {}", INDEX_JSON, e)?,
        };

        index.validate()?;

        let manifests = index.manifests();
        for desc in manifests {
            self.import_descriptor(desc, layout_index)?;
        }

        Ok(())
    }

    fn import_manifest(&mut self, path: &Path) -> RegistryResult<()>
    {
        let manifest = match ImageManifest::from_file(path) {
            Ok(i) => i,
            Err(e) => err!("Error importing \"{}\": {}", INDEX_JSON, e)?,
        };

        manifest.validate()?;

        let config = manifest.config();
        self.import_descriptor(config, false)?;

        let layers = manifest.layers();
        for desc in layers {
            self.import_descriptor(desc, false)?;
        }

        Ok(())
    }

    fn new(path: &Path) -> Self
    {
        let mut app = Application::default();
        app.path = path.to_owned();
        app
    }

    pub fn import(path: &Path) -> RegistryResult<Self>
    {
        info!("Loading application from: \"{}\"", path.display());

        let mut app = Application::new(path);

        let oci_layout = match OciLayout::from_file(path.join(OCI_LAYOUT)) {
            Ok(l) => l,
            Err(e) => err!("Error importing \"{}\": {}", OCI_LAYOUT, e)?,
        };

        oci_layout.validate()?;

        let index_path = path.join(INDEX_JSON);
        app.import_index(&index_path, true)?;

        let orphans = app.find_orphans()?;
        if !orphans.is_empty() {
            warn!("Found orphaned files: {:#?}", orphans);
        }

        Ok(app)
    }
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
