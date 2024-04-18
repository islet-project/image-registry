pub mod async_client;
pub mod error;

mod service_url;

use error::Error;
use log::{debug, error, info};
use ir_protocol::Manifest;
use reqwest::blocking::{Client as ReqwestClient, Response};
use service_url::{ServiceFile, ServiceUrl};
use std::fs::File;
use std::io::{BufWriter, Error as IOError};
use url::Url;
use uuid::Uuid;

pub struct Client {
    url: ServiceUrl,
    reqwest_client: ReqwestClient,
}

impl Client {
    /// Create new client for communication with server with given url.
    pub fn new(host: String) -> Self {
        Self {
            url: ServiceUrl::new(host),
            reqwest_client: ReqwestClient::new(),
        }
    }

    /// Fetch Manifest with given Uuid.
    pub fn get_manifest(&self, uuid: Uuid) -> Result<Manifest, Error> {
        let response = self.get_response(self.url.get_url(ServiceFile::ImageManifest(uuid))?)?;

        match response
            .json::<Manifest>()
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

    pub fn get_and_save_manifest(&self, uuid: Uuid, path: Option<String>) -> Result<(), Error> {
        let mut response = self
            .get_response(self.url.get_url(ServiceFile::ImageManifest(uuid))?)
            .inspect_err(|e| error!("Failed to get response {:?}", e))?;

        let filename = Self::conclude_path(ServiceFile::ImageManifest(uuid), path);
        info!("Saving manifest to {}", filename);
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        response
            .copy_to(&mut writer)
            .map_err(|_| Error::IOError(IOError::last_os_error()))?;

        Ok(())
    }

    pub fn get_image(&self, uuid: Uuid) -> Result<Vec<u8>, Error> {
        let response = self.get_response(self.url.get_url(ServiceFile::ImageArchive(uuid))?)?;
        Ok(response.bytes().map_err(|_| Error::UnknownError)?.to_vec())
    }

    /// Fetch Image with given Uuid and save to filename.
    pub fn get_and_save_image(&self, uuid: Uuid, path: Option<String>) -> Result<(), Error> {
        let mut response = self
            .get_response(self.url.get_url(ServiceFile::ImageArchive(uuid))?)
            .inspect_err(|e| error!("Failed to get response: {:?}", e))?;
        // let mut response = self.get_response(Url::parse(
        //     "https://ftp.gnu.org/gnu/binutils/binutils-2.6-2.7.patch.gz").unwrap())?;

        let filename = Self::conclude_path(ServiceFile::ImageArchive(uuid), path);
        info!("Saving image to {}", filename);
        let file = File::create(filename)?;
        let mut writer = BufWriter::new(file);

        response
            .copy_to(&mut writer)
            .map_err(|_| Error::IOError(IOError::last_os_error()))?;

        Ok(())
    }

    pub fn conclude_path(file: ServiceFile, path: Option<String>) -> String {
        match path {
            Some(filename) => filename,
            None => file.get_filename(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wrong_host() {
        let client = Client::new("http://serverdoesnot.exist".to_string());

        let uuid = Uuid::new_v4();
        assert!(matches!(
            client.get_manifest(uuid.clone()),
            Err(Error::ConnectionError)
        ));

        assert!(matches!(
            client.get_and_save_image(uuid.clone(), None),
            Err(Error::ConnectionError)
        ));
    }
}