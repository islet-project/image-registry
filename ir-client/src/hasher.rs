use std::fmt::Display;

use pin_project::pin_project;
use sha2::{digest::DynDigest, Digest, Sha256, Sha512};
use tokio::io::AsyncRead;
use crate::oci::reference::{SHA_256, SHA_512};

#[derive(Clone, Debug)]
pub enum HashType {
    Sha256,
    Sha512
}

impl Display for HashType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sha256 => f.write_str(SHA_256),
            Self::Sha512 => f.write_str(SHA_512)
        }
    }
}

#[pin_project]
pub struct Hasher<T: AsyncRead + Unpin> {
    hash: Box<dyn DynDigest + Send>,

    #[pin]
    inner: T,
}

impl<T: AsyncRead + Unpin> Hasher<T> {
    pub fn new(ty: &HashType, inner: T) -> Self {
        let hash = match ty {
            HashType::Sha256 => Box::new(Sha256::new()) as Box<dyn DynDigest + Send>,
            HashType::Sha512 => Box::new(Sha512::new()) as Box<dyn DynDigest + Send>
        };

        Self {
            hash,
            inner,
        }
    }

    pub fn finalize(&mut self) -> Box<[u8]> {
        self.hash.finalize_reset()
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for Hasher<T> {
    fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
        let this = self.project();

        let previous = buf.filled().len();
        match this.inner.poll_read(cx, buf) {
            std::task::Poll::Pending => std::task::Poll::Pending,

            std::task::Poll::Ready(v) => {
                let data = &buf.filled()[previous..];
                this.hash.update(data);
                std::task::Poll::Ready(v)
            }
        }
    }
}
