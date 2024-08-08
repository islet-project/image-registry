use crate::config::Config;
use crate::error::Error;
use crate::reference::{Digest, Reference, Tag};
use crate::service_url::{ServiceFile, ServiceUrl, TagList};

use std::any::type_name;
use std::io::Read;

use log::{debug, error, info};
use oci_spec::distribution::TagList as OciTagList;
use oci_spec::image::ImageManifest as OciImageManifest;
use reqwest::blocking::{Client as ReqwestClient, Response};
use serde::de::DeserializeOwned;
use url::Url;

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
        let request_url = self.url.get_url_path(app_name, ServiceFile::Manifest(reference))?;
        let response = self
            .get_response(request_url)
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;

        Self::extract_json(response)
    }

    pub fn get_blob_reader(&self, app_name: &str, digest: Digest) -> Result<impl Read, Error> {
        let request_url = self.url.get_url_path(app_name, ServiceFile::Blob(digest))?;
        let response = self
            .get_response(request_url)
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;
        Ok(response)
    }

    pub fn list_tags(&self, app_name: &str) -> Result<OciTagList, Error> {
        let request_url = self.url.get_url_path(app_name, ServiceFile::TagList(TagList::new()))?;
        let response = self
            .get_response(request_url)
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;

        Self::extract_json(response)
    }

    pub fn list_tags_with_options(&self, app_name: &str, n: Option<usize>, last: Option<Tag>) -> Result<OciTagList, Error> {
        let tag_list = TagList::with_options(n, last);

        let request_url = self.url.get_url_path(app_name, ServiceFile::TagList(tag_list))?;
        let response = self
            .get_response(request_url)
            .inspect_err(|e| error!("Failed to get response: {:?}, e", e))?;

        Self::extract_json(response)
    }

    fn extract_json<T: DeserializeOwned>(response: Response) -> Result<T, Error> {
        match response.json::<T>() {
            Ok(value) => Ok(value),
            Err(err) => {
                error!("Failed to parse {} as JSON", type_name::<T>());
                if err.is_decode() {
                    Err(Error::JSONParsingError(err.to_string()))
                } else {
                    Err(Error::UnknownError)
                }
            }
        }
    }

    fn get_response(&self, url: Url) -> Result<Response, Error> {
        info!("Fetching response from {}", url);
        match self
            .reqwest_client
            .get(url)
            .send()
            .inspect_err(|e| error!("Failed to send request: {}", e))
        {
            Ok(response) => {
                if response.status().is_success() {
                    debug!(
                        "Content type: {:?}",
                        response
                            .headers()
                            .get(reqwest::header::CONTENT_TYPE)
                            .unwrap()
                    );

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
