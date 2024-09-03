use crate::error::Error;
use crate::oci::reference::{Digest, Reference, Tag};

use oci_spec::image::MediaType;
use url::Url;

pub(crate) struct TagList {
    n: Option<usize>,
    last: Option<Tag>,
}

impl TagList {
    const LIST: &'static str = "list";
    const N_QUERY: &'static str = "n";
    const LAST_QUERY: &'static str = "last";

    pub fn as_str(&self) -> &str {
        Self::LIST
    }

    pub(crate) fn options(&self) -> Option<Vec<(String, String)>>{
        let mut options = Vec::new();
        if let Some(n) = &self.n {
            options.push((Self::N_QUERY.to_string(), n.to_string()));
        }
        if let Some(last) = &self.last {
            options.push((Self::LAST_QUERY.to_string(), last.as_str().to_string()));
        }
        if options.is_empty() {
            None
        } else {
            Some(options)
        }
    }

    pub fn new() -> Self {
        Self { n: None, last: None }
    }

    pub fn with_options(n: Option<usize>, last: Option<Tag>) -> Self {
        Self { n, last }
    }
}

pub(crate) enum ServiceFile {
    Manifest(Reference),
    Blob(Digest),
    TagList(TagList),
}

impl ServiceFile {
    const MANIFEST_PATH: &'static str = "manifests/";
    const BLOBS_PATH: &'static str = "blobs/";
    const TAGS_PATH: &'static str = "tags/";

    pub fn get_file_uri(&self) -> String {
        match self {
            Self::Manifest(reference) => format!("{}{}", Self::MANIFEST_PATH, reference.to_string()),
            Self::Blob(digest) => format!("{}{}", Self::BLOBS_PATH, digest.to_string()),
            Self::TagList(tag_list) => format!("{}{}", Self::TAGS_PATH, tag_list.as_str()),
        }
    }

    pub fn get_options(&self) -> Option<Vec<(String, String)>> {
        if let Self::TagList(tag_list) = self {
            return tag_list.options();
        }

        None
    }

    pub fn supported_media_types(&self) -> Vec<String> {
        match self {
            Self::Manifest(_) => vec![MediaType::ImageManifest.to_string()],
            Self::Blob(_) => vec![
                MediaType::ImageLayer.to_string(),
                MediaType::ImageLayerGzip.to_string(),
                MediaType::ImageLayerZstd.to_string(),
                MediaType::ImageConfig.to_string()],
            Self::TagList(_) => vec![mime::APPLICATION_JSON.to_string()],
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
            return Ok(
                (scheme == &user_parsed.scheme())
                    .then_some(user_parsed)
                    .ok_or(Error::UrlParsingError(format!("Invalid user scheme: {}", user_scheme)))?
            );
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

        let mut url = self
            .base_url()?
            .join(app_name_path)?
            .join(&file.get_file_uri())?;

        if let Some(options) = file.get_options() {
            let mut append_queries = url.query_pairs_mut();
            for (query_name, query_value) in options {
                append_queries.append_pair(&query_name, &query_value);
            }
        }

        Ok(url)
    }
}
