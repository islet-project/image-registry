use crate::error::Error;
use url::Url;
use uuid::Uuid;

pub struct ServiceUrl {
    host: String,
}

pub enum ServiceFile {
    ImageManifest(Uuid),
    ImageArchive(Uuid),
}

impl ServiceFile {
    const MANIFEST_FILE_EXT: &'static str = "json";
    const IMAGE_FILE_EXT: &'static str = "tgz";

    pub fn get_filename(&self) -> String {
        match self {
            Self::ImageArchive(uuid) => format!("{}.{}", uuid.to_string(), Self::IMAGE_FILE_EXT),
            Self::ImageManifest(uuid) => {
                format!("{}.{}", uuid.to_string(), Self::MANIFEST_FILE_EXT)
            }
        }
    }
}

pub const HOST_PROTOCOL_SECURE_PREFIX: &'static str = "https://";
pub const HOST_PROTOCOL_NONSECURE_PREFIX: &'static str = "http://";

impl ServiceUrl {
    const REGISTRY_PATH: &'static str = "image/";

    // Fix: parse host here
    pub fn from_str(host: String) -> Self {
        Self { host }
    }

    pub fn get_url(&self, file: ServiceFile) -> Result<Url, Error> {
        let url =
            Url::parse(&self.host).inspect_err(|e| println!("Failed to parse host: {}", e))?;
        Ok(url.join(Self::REGISTRY_PATH)?.join(&file.get_filename())?)
    }
}
