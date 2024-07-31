use crate::error::Error;
use url::Url;
use uuid::Uuid;

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

pub(crate) const HOST_PROTOCOL_SECURE_SCHEME: &'static str = "https://";
pub(crate) const HOST_PROTOCOL_NONSECURE_SCHEME: &'static str = "http://";

pub struct ServiceUrl {
    scheme:String,
    host: String,
}

impl ServiceUrl {
    const REGISTRY_PATH: &'static str = "image/";

    /// Scheme should be "http://" or "https://".
    /// Use Config::scheme() to get proper scheme for your connection.
    pub fn init(scheme: String, host: String) -> Self {
        Self { scheme, host }
    }

    fn host(&self) -> String {
        self.scheme.clone() + &self.host

    }

    pub fn get_url(&self, file: ServiceFile) -> Result<Url, Error> {
        let url =
            Url::parse(&self.host()).inspect_err(|e| println!("Failed to parse host: {}", e))?;
        Ok(url.join(Self::REGISTRY_PATH)?.join(&file.get_filename())?)
    }
}
