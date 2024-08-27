use pin_project::pin_project;
use sha2::{digest::DynDigest, Digest, Sha256, Sha512};
use std::future::Future;
use std::io::Result;
use std::path::Path;
use std::task::{ready, Poll};
use tokio::io::{AsyncRead, ReadBuf};
use tokio::pin;

use super::digest::Digest as OciDigest;
use crate::error::RegistryError;
use crate::RegistryResult;

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistry(format!($($arg)+))))
}

#[pin_project]
pub struct Hasher<T: AsyncRead>
{
    hash: Box<dyn DynDigest>,

    #[pin]
    inner: T,
}

impl<T: AsyncRead> Hasher<T>
{
    pub fn new(digest: &OciDigest, inner: T) -> RegistryResult<Self>
    {
        let hash = match digest.get_algo() {
            "sha256" => Box::new(Sha256::new()) as Box<dyn DynDigest>,
            "sha512" => Box::new(Sha512::new()) as Box<dyn DynDigest>,
            a => err!("Wrong hash algorithm: {}", a)?,
        };

        Ok(Self { hash, inner })
    }

    pub fn finalize(&mut self) -> Box<[u8]>
    {
        self.hash.finalize_reset()
    }
}

impl<T: AsyncRead> Future for Hasher<T>
{
    type Output = Result<Option<Vec<u8>>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output>
    {
        let this = self.project();
        let mut buf_int = [0 as u8; 8192];
        let mut buf = ReadBuf::new(&mut buf_int);

        match ready!(this.inner.poll_read(cx, &mut buf)) {
            Ok(()) => {
                let data = buf.filled();
                if data.len() == 0 {
                    Poll::Ready(Ok(Some(this.hash.finalize_reset().to_vec())))
                } else {
                    this.hash.update(data);
                    Poll::Ready(Ok(None))
                }
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

pub async fn verify<P: AsRef<Path>>(path: P, digest: &OciDigest) -> RegistryResult<bool>
{
    let file = tokio::fs::File::open(path).await?;
    let hasher = Hasher::new(digest, file)?;

    pin!(hasher);

    let hash = loop {
        if let Some(hash) = hasher.as_mut().await? {
            println!("{:x?}", hash);
            break hash;
        }
    };

    // unwrap() below is intentional, new() for digest will make sure the hash
    // is correct, verifying digest created with new_unchecked() is an error
    if hash == hex::decode(digest.get_hash()).unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}
