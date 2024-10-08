use log::debug;

use oci_spec::image::MediaType;
use reqwest::header::{HeaderMap, HeaderName, CONTENT_LENGTH, CONTENT_TYPE};
use sha2::{Digest as Sha2Digest, Sha256, Sha512};

use crate::{hasher::HashType, oci::reference::Digest};

pub(crate) fn content_type(headers: &HeaderMap) -> Option<String> {
    headers
        .get(CONTENT_TYPE)
        .and_then(|ct| ct.to_str().ok().map(|ct| {
            debug!("Content-type: {ct}");
            ct.to_string()
        }))
}

pub(crate) fn content_length(headers: &HeaderMap) -> Option<usize> {
    headers
        .get(CONTENT_LENGTH)
        .and_then(|cl| cl.to_str().ok().map(|cl| {
            debug!("Content-length: {cl}");
            cl.parse().ok()
        }))?
}

pub(crate) fn docker_content_digest(headers: &HeaderMap) -> Option<String> {
    headers
        .get(HeaderName::from_static("docker-content-digest"))
        .and_then(|cd| cd.to_str().ok().map(|cd| {
            debug!("docker-content-digest: {cd}");
            cd.to_string()
        }))
}

pub(crate) fn verify_content_type(content_type: &Option<String>, media_type: &Option<MediaType>) -> bool {
    if let (Some(ct), Some(mt)) = (content_type, media_type) {
        ct == &mt.to_string()
    } else {
        true
    }
}

pub fn verify_digest(digest: &Digest, content: &[u8]) -> bool {
    let digest_value = hex::decode(digest.value()).unwrap_or_default();

    match digest.hash_type() {
        HashType::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(content);
            let hash = hasher.finalize();
            debug!("Computed sha256: {}", hex::encode(hash));
            hash.as_slice() == digest_value.as_slice()
        },
        HashType::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(content);
            let hash = hasher.finalize();
            debug!("Computed sha512: {}", hex::encode(hash));
            hash.as_slice() == digest_value.as_slice()
        },
    }
}
