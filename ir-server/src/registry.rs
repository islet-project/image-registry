use async_trait::async_trait;
use tokio::fs;
use tokio_util::io;

#[allow(dead_code)]
#[async_trait]
pub trait ImageRegistry: Send + Sync
{
    fn get_tags(&self, app: &str) -> Vec<String>;
    async fn get_manifest(&self, app: &str, reference: &str) -> Option<io::ReaderStream<fs::File>>;
    async fn get_blob(&self, app: &str, digest: &str) -> Option<io::ReaderStream<fs::File>>;
}
