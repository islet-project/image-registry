use log::debug;

use oci_spec::image::MediaType;
use reqwest::header::{HeaderMap, HeaderName, CONTENT_LENGTH, CONTENT_TYPE};
use sha2::{Digest as Sha2Digest, Sha256, Sha512};

use crate::{hasher::HashType, reference::Digest};

pub(crate) fn content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(CONTENT_TYPE)
        .map(|ct| ct.to_str().ok().map(|ct| ct.to_string())).flatten()
}

pub(crate) fn content_length(headers: &HeaderMap) -> Option<usize> {
    headers
        .get(CONTENT_LENGTH)
        .map(|cl| cl.to_str().ok().map(|cl| cl.parse().ok())).flatten()?
}

pub(crate) fn docker_content_digest(headers: &HeaderMap) -> Option<String> {
    headers
        .get(HeaderName::from_static("docker-content-digest"))
        .map(|ct| ct.to_str().ok().map(|ct| ct.to_string())).flatten()
}

pub(crate) fn verify_content_type(content_type: &Option<String>, media_type: &Option<MediaType>) -> bool {
    if let (Some(ct), Some(mt)) = (content_type, media_type) {
        ct == &mt.to_string()
    } else {
        true
    }
}

pub fn verify_digest(digest: &Digest, content: &[u8]) -> bool {
    let digest_value = hex::decode(digest.value()).unwrap_or(Vec::new());

    match digest.hash_type() {
        HashType::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(content);
            let hash = hasher.finalize();
            debug!("Computed sha256: {}", hex::encode(&hash));
            hash.as_slice() == digest_value.as_slice()
        },
        HashType::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(content);
            let hash = hasher.finalize();
            debug!("Computed sha512: {}", hex::encode(&hash));
            hash.as_slice() == digest_value.as_slice()
        },
    }
}
