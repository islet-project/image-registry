use log::debug;
use oci_spec::image::MediaType;
use reqwest::header::{HeaderMap, CONTENT_TYPE};

pub(crate) fn verify_content_type(content_type: &Option<String>, media_type: &Option<MediaType>) -> bool {
    if let (Some(ct), Some(mt)) = (content_type, media_type) {
        debug!("Media-type: \"{mt}\"");
        ct == &mt.to_string()
    } else {
        true
    }
}

pub(crate) fn content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(CONTENT_TYPE)
        .map(|ct| ct.to_str().ok().map(|ct| ct.to_string())).flatten()
}
