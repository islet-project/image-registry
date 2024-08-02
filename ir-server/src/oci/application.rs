use log::{info, warn};
use oci_spec::image::{
    Descriptor, ImageIndex, ImageManifest, MediaType, OciLayout, ANNOTATION_REF_NAME,
};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use super::digest::Digest;
use super::sha2;
use super::tag;
use super::validate::Validate;
use crate::error::RegistryError;
use crate::RegistryResult;

const OCI_LAYOUT: &str = "oci-layout";
const INDEX_JSON: &str = "index.json";
const BLOBS_SUBDIR: &str = "blobs";

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistry(format!($($arg)+))))
}

#[derive(Debug, Clone)]
pub(super) struct Content
{
    pub(super) path: PathBuf,
    pub(super) size: u64,
    pub(super) digest: String,
    pub(super) media_type: String,
}

#[derive(Debug, Default)]
pub(super) struct Application
{
    path: PathBuf,
    tags: HashMap<String, Content>,
    manifests: HashMap<Digest, Content>,
    blobs: HashMap<Digest, Content>,
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

                if let (None, None) = (self.blobs.get(&digest), self.manifests.get(&digest)) {
                    orphans.push(digest);
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

        let size: u64 = desc.size().try_into().or(err!("File size negative"))?;
        let file_size = std::fs::metadata(&path)?.len();
        if file_size != size {
            err!("Wrong file length: {}, expected: {}", file_size, size)?;
        }

        let media_type = desc.media_type();
        let content = Content {
            path: path.clone(),
            size,
            digest: digest.to_string(),
            media_type: media_type.to_string(),
        };

        // For layout index descriptors do two additional things:
        // - verify hashes of linked manifests
        // - load tags from annotations
        if layout_index {
            if !sha2::verify(&path, &digest)? {
                err!("SHA failed for \"{}\"", path.display())?;
            }

            if let Some(anns) = desc.annotations() {
                if let Some(tag) = anns.get(ANNOTATION_REF_NAME) {
                    if !tag::verify(tag) {
                        err!("Tag \"{}\" doesn't match: {}", tag, tag::PATTERN)?;
                    }
                    self.tags.insert(tag.to_string(), content.clone());
                }
            }
        }

        match media_type {
            MediaType::ImageIndex => {
                self.import_index(&path, false)?;
                self.manifests.insert(digest, content);
            }
            MediaType::ImageManifest => {
                self.import_manifest(&path)?;
                self.manifests.insert(digest, content);
            }
            MediaType::ImageConfig
            | MediaType::ImageLayer
            | MediaType::ImageLayerGzip
            | MediaType::ImageLayerZstd => {
                self.blobs.insert(digest, content);
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
        Self {
            path: path.to_owned(),
            ..Default::default()
        }
    }

    pub(super) fn get_tags(&self) -> &HashMap<String, Content>
    {
        &self.tags
    }

    pub(super) fn get_manifests(&self) -> &HashMap<Digest, Content>
    {
        &self.manifests
    }

    pub(super) fn get_blobs(&self) -> &HashMap<Digest, Content>
    {
        &self.blobs
    }

    pub(super) fn import(path: &Path) -> RegistryResult<Self>
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
