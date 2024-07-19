use async_trait::async_trait;
use ir_protocol::Manifest;
use log::info;
use tokio::fs;
use tokio_util::io;
use uuid::Uuid;

use crate::registry::ImageRegistry;
use crate::RegistryResult;

#[derive(Debug)]
pub struct Registry {}

impl Registry
{
    pub fn import(path: &str) -> RegistryResult<Self>
    {
        info!("Loading registry from: {}", path);

        Ok(Registry {})
    }
}

#[async_trait]
impl ImageRegistry for Registry
{
    fn get_manifest(&self, _uuid: &Uuid) -> Option<&Manifest>
    {
        None
    }

    async fn get_image(&self, _uuid: &Uuid) -> Option<io::ReaderStream<fs::File>>
    {
        None
    }
}
