use async_trait::async_trait;
use tokio::fs;
use tokio_util::io;

pub struct Payload
{
    pub stream: io::ReaderStream<fs::File>,
    pub size: u64,
    pub digest: String,
    pub media_type: String,
}

#[async_trait]
pub trait ImageRegistry: Send + Sync
{
    fn get_tags(&self, app: &str) -> Option<Vec<String>>;
    async fn get_manifest(&self, app: &str, reference: &str) -> Option<Payload>;
    async fn get_blob(&self, app: &str, digest: &str) -> Option<Payload>;
}
