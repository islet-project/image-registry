use crate::{
    config::Config, error::Error, reference::{Digest, Reference, Tag}, service_url::{ServiceFile, ServiceUrl, TagList}, utils
};
use futures::stream::TryStreamExt;
use log::{debug, error, info, warn};
use oci_spec::{distribution::TagList as OciTagList, image::{ImageManifest as OciImageManifest, MediaType}};
use reqwest::{header::ACCEPT, Client as ReqwestClient, Response};
use serde::de::DeserializeOwned;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

pub struct BlobStream {
    response: Option<Response>,
    length: Option<usize>,
    digest: Option<Digest>,
    media_type: Option<MediaType>,
}

impl BlobStream {
    pub fn from_response(response: Response) -> Result<Self, Error> {
        let content_type = utils::content_type(response.headers());
        let media_type = content_type.map(|ct| MediaType::from(ct.as_str()));
        let length = utils::content_length(response.headers());
        let content_digest = utils::docker_content_digest(response.headers());
        let digest = content_digest.map(
            |cd| Digest::try_from(cd.as_str())
                .map_err(|_| Error::ResponseDigestInvalid)
                .ok()
        ).flatten();

        Ok(Self { response: Some(response), length, media_type, digest })
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

    pub fn take_reader(&mut self) -> Option<impl AsyncRead> {
        let Some(response) = self.response.take() else {
            return None;
        };

        let stream = response.bytes_stream().map_err(std::io::Error::other);
        Some(StreamReader::new(stream))
    }
}

pub struct Client {
    url: ServiceUrl,
    reqwest_client: ReqwestClient,
}

impl Client {
    /// Create new image registry async client from given configuration
    pub fn from_config(config: Config) -> Result<Self, Error> {
        let Config {host, mode} = config;
        let url = ServiceUrl::init(mode.scheme(), host);
        match mode.into_rustls_config() {
            None => Ok (
                Self {
                    url,
                    reqwest_client: ReqwestClient::new(),
                }
            ),
            Some(client_config) => {
                let reqwest_client = ReqwestClient::builder()
                    .use_preconfigured_tls(client_config)
                    .build()
                    .map_err(Error::into_config)?;
                Ok (
                    Self {
                        url,
                        reqwest_client
                    }
                )
            }
        }
    }

    pub async fn get_manifest(&self, app_name: &str, reference: Reference) -> Result<OciImageManifest, Error> {
        let response = self
            .get_response(app_name, ServiceFile::Manifest(reference))
            .await?;

        let content_type = utils::content_type(response.headers());
        let manifest: OciImageManifest = Self::extract_json(response).await?;

        if !utils::verify_content_type(&content_type, manifest.media_type()) {
            warn!("Conent-type doesn't match media-type");
        }

        Ok(manifest)
    }

    pub async fn get_blob_stream(&self, app_name: &str, digest: Digest) -> Result<BlobStream, Error> {
        let response = self
            .get_response(app_name, ServiceFile::Blob(digest))
            .await?;

        BlobStream::from_response(response)
    }

    pub async fn list_tags(&self, app_name: &str) -> Result<OciTagList, Error> {
        let response = self
            .get_response(app_name, ServiceFile::TagList(TagList::new()))
            .await?;

        Self::extract_json(response).await
    }

    pub async fn list_tags_with_options(&self, app_name: &str, n: Option<usize>, last: Option<Tag>) -> Result<OciTagList, Error> {
        let tag_list = TagList::with_options(n, last);

        let response = self
            .get_response(app_name, ServiceFile::TagList(tag_list))
            .await?;

        Self::extract_json(response).await
    }

    async fn extract_json<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
        let content_length = utils::content_length(response.headers());
        let content_digest = utils::docker_content_digest(response.headers());

        let bytes = response.bytes().await.map_err(|_| Error::UnknownError)?;

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

        serde_json::from_slice(&bytes).map_err(|e| Error::JSONParsingError(e.to_string()))
    }

    async fn get_response(&self, app_name: &str, file: ServiceFile) -> Result<Response, Error> {
        let accepted_types = file.supported_media_types();
        let url = self.url.get_url_path(app_name, file)?;

        info!("Fetching response from {}", url);
        debug!("Supported media types: {}", accepted_types.join(","));

        match self
            .reqwest_client
            .get(url)
            .header(ACCEPT, accepted_types.join(","))
            .send()
            .await
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
