use crate::config::Config;
use crate::error::Error;
use crate::reference::{Digest, Reference};
use crate::service_url::{ServiceFile, ServiceUrl};

use std::io::Read;

use log::{debug, error, info};
use oci_spec::image::ImageManifest;
use reqwest::blocking::{Client as ReqwestClient, Response};
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

    pub fn get_manifest(&self, app_name: &str, reference: Reference) -> Result<ImageManifest, Error> {
        let request_url = self.url.get_url_path(app_name, ServiceFile::Manifest(reference))?;
        let response = self
            .get_response(request_url)
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;

        match response
            .json::<ImageManifest>()
            .inspect_err(|e| error!("Failed  to parse JSON: {}", e))
        {
            Ok(manifest) => Ok(manifest),
            Err(err) => {
                if err.is_decode() {
                    Err(Error::JSONParsingError(err.to_string()))
                } else {
                    Err(Error::UnknownError)
                }
            }
        }
    }

    pub fn get_blob_reader(&self, app_name: &str, digest: Digest) -> Result<impl Read, Error> {
        let request_url = self.url.get_url_path(app_name, ServiceFile::Blob(digest))?;
        let response = self
            .get_response(request_url)
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;
        Ok(response)
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
