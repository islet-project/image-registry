use crate::{
    error::Error,
    service_url::{ServiceFile, ServiceUrl},
};
use futures::stream::TryStreamExt;
use log::{debug, error, info};
use ir_protocol::Manifest;
use reqwest::{Client as ReqwestClient, Response};
use url::Url;
use uuid::Uuid;

pub struct Client {
    url: ServiceUrl,
    reqwest_client: ReqwestClient,
}

impl Client {
    pub fn new(host: String) -> Self {
        Self {
            url: ServiceUrl::init("http://".to_string(), host),
            reqwest_client: ReqwestClient::new(),
        }
    }

    pub async fn get_manifest(&self, uuid: Uuid) -> Result<Manifest, Error> {
        let response = self
            .get_response(self.url.get_url(ServiceFile::ImageManifest(uuid))?)
            .await?;

        match response
            .json::<Manifest>()
            .await
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

    pub async fn get_image_stream(&self, uuid: Uuid) -> Result<impl tokio::io::AsyncRead, Error> {
        let response = self
            .get_response(self.url.get_url(ServiceFile::ImageArchive(uuid))?)
            .await?;

        let stream = response.bytes_stream().map_err(std::io::Error::other);
        Ok(tokio_util::io::StreamReader::new(stream))
    }

    async fn get_response(&self, url: Url) -> Result<Response, Error> {
        info!("Fetching response from {}", url);
        match self
            .reqwest_client
            .get(url)
            .send()
            .await
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
