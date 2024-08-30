use crate::config::Config;
use crate::error::Error;
use crate::oci::reference::{Digest, Reference, Tag};
use crate::oci::service_url::{ServiceFile, ServiceUrl, TagList};
use crate::utils;

use std::io::Read;

use log::{debug, error, info, warn};
use oci_spec::distribution::TagList as OciTagList;
use oci_spec::image::{ImageManifest as OciImageManifest, MediaType};
use reqwest::blocking::{Client as ReqwestClient, Response};
use reqwest::header::ACCEPT;
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub struct BlobReader {
    response: Response,
    length: Option<usize>,
    media_type: Option<MediaType>,
    digest: Option<Digest>
}

impl BlobReader {
    pub fn from_response(response: Response) -> Result<Self, Error> {
        let content_type = utils::content_type(response.headers());
        let media_type = content_type.map(|ct| MediaType::from(ct.as_str()));
        let length = utils::content_length(response.headers());
        let content_digest = utils::docker_content_digest(response.headers());
        let digest = content_digest.and_then(
            |cd| Digest::try_from(cd.as_str())
                .map_err(|_| Error::ResponseDigestInvalid)
                .ok()
        );
        Ok(Self { response, length, media_type, digest })
    }

    pub fn len(&self) -> &Option<usize> {
        &self.length
    }

    pub fn media_type(&self) -> &Option<MediaType> {
        &self.media_type
    }

    pub fn digest(&self) -> &Option<Digest> {
        &self.digest
    }
}

impl Read for BlobReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Consider wrapping errors, so http errors do not bleed out
        self.response.read(buf)
    }
}

pub struct Client {
    url: ServiceUrl,
    reqwest_client: ReqwestClient,
}

impl Client {
    /// Create new image registry client from given configuration
    pub fn from_config(config: Config) -> Result<Self, Error> {
        let Config {host, mode} = config;
        let url = ServiceUrl::init(mode.scheme(), host);
        match mode.into_rustls_config() {
            None => Ok(Self {
                url,
                reqwest_client: ReqwestClient::new(),
            }),
            Some(client_config) => {
                let reqwest_client = ReqwestClient::builder()
                        .use_preconfigured_tls(client_config)
                        .build()
                    .map_err(Error::into_config)?;
                Ok(Self {
                    url,
                    reqwest_client,
                })
            }
        }
    }

    pub fn get_manifest(&self, app_name: &str, reference: Reference) -> Result<OciImageManifest, Error> {
        let response = self
            .get_response(app_name, ServiceFile::Manifest(reference))
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;

        let content_type = utils::content_type(response.headers());
        // let manifest: OciImageManifest = Self::extract_json(response)?;
        let manifest: OciImageManifest = Self::extract_json(response)?;

        if !utils::verify_content_type(&content_type, manifest.media_type()) {
            warn!("Conent-type doesn't match media-type");
        }

        Ok(manifest)
    }

    pub fn get_blob_reader(&self, app_name: &str, digest: Digest) -> Result<BlobReader, Error> {
        let response = self
            .get_response(app_name, ServiceFile::Blob(digest))
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;
        BlobReader::from_response(response)
    }

    pub fn list_tags(&self, app_name: &str) -> Result<OciTagList, Error> {
        let response = self
            .get_response(app_name, ServiceFile::TagList(TagList::new()))
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;

        Self::extract_json(response)
    }

    pub fn list_tags_with_options(&self, app_name: &str, n: Option<usize>, last: Option<Tag>) -> Result<OciTagList, Error> {
        let tag_list = TagList::with_options(n, last);

        let response = self
            .get_response(app_name, ServiceFile::TagList(tag_list))
            .inspect_err(|e| error!("Failed to get response: {:?}, e", e))?;

        Self::extract_json(response)
    }

    fn extract_json<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
        let content_length = utils::content_length(response.headers());
        let content_digest = utils::docker_content_digest(response.headers());

        let bytes = response.bytes().map_err(|_| Error::UnknownError)?;

        if let Some(cl) = content_length {
            debug!("Content-Length: {cl}");
            if cl != bytes.len() {
                error!("Response length doesn't match servers content-length");
                return Err(Error::ResponseLengthInvalid);
            }
        }

        if let Some(cd) = content_digest {
            debug!("Docker-content-digest: {cd}");
            let digest = Digest::try_from(cd.as_str()).map_err(|_| Error::ResponseDigestInvalid)?;
            if !utils::verify_digest(&digest, &bytes) {
                error!("Response digest doesn't match servers docker-content-digest");
                return Err(Error::ResponseDigestInvalid)?;
            }
        }

        Ok(serde_json::from_slice(&bytes)?)
    }

    fn get_response(&self, app_name: &str, file: ServiceFile) -> Result<Response, Error> {
        let accepted_types = file.supported_media_types();
        let url = self.url.get_url_path(app_name, file)?;

        info!("Fetching response from {}", url);
        debug!("Supported media types: {}", accepted_types.join(","));

        match self
            .reqwest_client
            .get(url)
            .header(ACCEPT, accepted_types.join(","))
            .send()
            .inspect_err(|e| error!("Failed to send request: {}", e))
        {
            Ok(response) => {
                if response.status().is_success() {
                    if let Some(content_type_str) = utils::content_type(response.headers()) {
                        debug!("Content-Type:\"{content_type_str}\"");

                        if !accepted_types.contains(&content_type_str.to_string()) {
                            warn!("Server returned unsupported content type");
                        }
                    }

                    Ok(response)
                } else {
                    Err(Error::StatusError(response.status().as_u16()))
                }
            }
            Err(err) => {
                if let Some(status_error) = err.status() {
                    Err(Error::StatusError(status_error.as_u16()))
                } else {
                    Err(Error::ConnectionError)
                }
            }
        }
    }
}
