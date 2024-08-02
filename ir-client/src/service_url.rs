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
// Accept "localhost:1337" the same as "https://localhost:1337" or "http://localhost:1337".
// Correct scheme will be added, when not passed by user. Otherwise it will be validated.
fn make_url(user: &str, scheme: &Scheme) -> Result<Url, Error> {
    if let Ok(user_parsed) = Url::parse(user) {
        // Otherwise scheme will be empty
        if user_parsed.has_host() {
            let user_scheme = user_parsed.scheme().to_owned();
            (scheme == &user_parsed.scheme())
                .then_some(user_parsed)
                .ok_or(Error::UrlParsingError(format!("Invalid user scheme: {}", user_scheme)))?;
        }
    }

    Ok(Url::parse(&format!("{}{}", scheme.as_str(), user))?)
}

pub(crate) struct ServiceUrl {
    scheme: &'static Scheme,
    host: String,
}

impl ServiceUrl {
    const VERSION_PATH: &'static str = "v2/";

    pub fn init(scheme: &'static Scheme, host: String) -> Self {
        Self { scheme, host }
    }

    fn base_url(&self) -> Result<Url, Error> {
       Ok(make_url(&self.host, self.scheme)?.join(Self::VERSION_PATH)?)

    }

    pub fn get_url_path(&self, app_name: &str, file: ServiceFile) -> Result<Url, Error> {
        let app_name_path = &format!("{}/", app_name);

        let url = self
            .base_url()?
            .join(app_name_path)?
            .join(&file.get_file_uri())?;

        Ok(url)
    }
}
