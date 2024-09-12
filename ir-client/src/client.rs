use std::collections::HashMap;
use std::path::Path;

use log::{error, info};
use oci_spec::image::{Descriptor, ImageConfiguration as OciConfig, MediaType};
use tokio::fs::{remove_file, File};
use tokio::io::{copy, AsyncReadExt, AsyncWriteExt};

use crate::config::Config;
use crate::error::Error;
use crate::layer::Image;
use crate::oci::client::Client as OciClient;
use crate::oci::reference::{Digest, Reference};
use crate::verify_digest;

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub(crate) app_name: String,
    pub(crate) layers: Vec<Descriptor>,
    config_digest: Digest,
    config: OciConfig,
    annotations: Option<HashMap<String, String>>,
}

impl ImageInfo {
    pub fn config(&self) -> &OciConfig {
        &self.config
    }
    pub fn config_digest(&self) -> &Digest {
        &self.config_digest
    }
    pub fn annotations(&self) -> Option<&HashMap<String, String>> {
        self.annotations.as_ref()
    }
}

pub struct Client {
    oci_client: OciClient,
}

impl Client {
    pub fn from_config(config: Config) -> Result<Self, Error> {
        Ok(Self {
            oci_client: OciClient::from_config(config)?,
        })
    }

    pub async fn get_image_info(
        &self,
        app_name: &str,
        reference: Reference,
    ) -> Result<ImageInfo, Error> {
        let manifest = self.oci_client.get_manifest(app_name, reference).await?;
        let config_digest_str = manifest.config().digest();
        let config_digest = Digest::try_from(config_digest_str.as_str())?;

        let mut config_reader = self
            .oci_client
            .get_blob_reader(app_name, config_digest.clone())
            .await?;
        let mut config_bytes = Vec::new();
        config_reader.read_to_end(&mut config_bytes).await?;

        if !verify_digest(&config_digest, &config_bytes) {
            error!("Digest of config returned by server differs from manifest");
            return Err(Error::DigestInvalidError);
        }

        let config: OciConfig = serde_json::from_slice(&config_bytes)?;

        Ok(ImageInfo {
            app_name: app_name.to_string(),
            config,
            config_digest,
            annotations: manifest.annotations().clone(),
            layers: manifest.layers().clone(),
        })
    }

    pub async fn unpack_image(
        &self,
        image_info: &ImageInfo,
        dest: impl AsRef<Path>,
        temp: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let image = Image::init(dest.as_ref());
        for (i, layer) in image_info.layers.iter().enumerate() {
            let layer_digest = Digest::try_from(layer.digest().as_str())?;
            let mut layer_reader = self
                .oci_client
                .get_blob_reader(&image_info.app_name, layer_digest)
                .await?;

            let extension = match layer.media_type() {
                MediaType::ImageLayer => "tar",
                MediaType::ImageLayerGzip => "tar.gz",
                MediaType::ImageLayerZstd => "tar.zstd",
                _ => return Err(Error::UnknownError),
            };

            let file_name = "layer_".to_string() + i.to_string().as_str();
            let layer_path = temp.as_ref().join(file_name).with_extension(extension);

            let mut layer_file = File::create(layer_path.clone()).await?;
            copy(&mut layer_reader, &mut layer_file).await?;

            layer_file.flush().await?;

            let diff_id = image_info
                .config
                .rootfs()
                .diff_ids()
                .get(i)
                .ok_or_else(|| {
                    error!("Not enough diff_ids for layers");
                    Error::LayerInvalidDiffIdError
                })?;

            let diff_id_digest = Digest::try_from(diff_id.as_str())?;

            info!(
                "Unpacking layer {} onto {}",
                layer_path.as_path().display(),
                dest.as_ref().display()
            );
            image
                .unpack_layer(&layer_path, layer.media_type(), diff_id_digest)
                .await?;

            remove_file(layer_path).await?;
        }
        Ok(())
    }
}
