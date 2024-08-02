use crate::error::Error;
use crate::reference::{Digest, Reference};

use url::Url;

pub(crate) enum ServiceFile {
    Manifest(Reference),
    Blob(Digest),
}

impl ServiceFile {
    const MANIFEST_PATH: &'static str = "manifests/";
    const BLOBS_PATH: &'static str = "blobs/";

    pub fn get_file_uri(&self) -> String {
        match self {
            Self::Manifest(reference) => format!("{}{}", Self::MANIFEST_PATH, reference.as_str()),
            Self::Blob(digest) => format!("{}{}", Self::BLOBS_PATH, digest.as_str()),
        }
    }
}

pub(crate) struct Scheme {
    scheme: &'static str,
}

impl Scheme {
    pub const fn init(scheme: &'static str) -> Self {
        Self {
            scheme
        }
    }

    pub const fn as_str(&self) -> &'static str {
        self.scheme
    }
}

pub(crate) const HTTPS_SCHEME: Scheme = Scheme::init("https://");
pub(crate) const HTTP_SCHEME: Scheme = Scheme::init("http://");

impl PartialEq<&str> for Scheme {
    fn eq(&self, other: &&str) -> bool {
        &self.scheme == other || &self.scheme.split_once(':').unwrap().0 == other
    }
}

fn make_url(user: &str, scheme: &Scheme) -> Result<Url, &'static str> {
    match Url::parse(user) {
        Ok(user_parsed) => {
            if user_parsed.has_host() {
                if scheme == &user_parsed.scheme() {
                    Ok(user_parsed)
                } else {
                    Err("User passed improper scheme")
                }
            } else {
                Url::parse(&format!("{}{}", scheme.as_str(), user)).map_err(|_| "failed to make url")
            }
        },
        Err(_) => {
            Url::parse(&format!("{}{}", scheme.as_str(), user)).map_err(|_| "failed to make url")
        }
    }
}

pub(crate) struct ServiceUrl {
    scheme: &'static Scheme,
    host: String,
}

impl ServiceUrl {
    const VERSION_PATH: &'static str = "v2/";

    /// Scheme should be "http://" or "https://".
    /// Use Config::scheme() to get proper scheme for connection.
    pub fn init(scheme: &'static Scheme, host: String) -> Self {
        Self { scheme, host }
    }

    fn host_url(&self) -> Url {
        make_url(&self.host, &self.scheme).unwrap()

    }

    pub fn get_url_path(&self, app_name: &str, file: ServiceFile) -> Result<Url, Error> {
        let app_name_path = &format!("{}/", app_name);
        Ok(self.host_url().join(Self::VERSION_PATH)?.join(&app_name_path)?.join(&file.get_file_uri())?)
    }
}
