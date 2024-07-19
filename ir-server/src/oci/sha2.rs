use sha2::{Digest, Sha256, Sha512};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::Digest as OciDigest;
use crate::error::RegistryError;
use crate::RegistryResult;

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistryError(format!($($arg)+))))
}

pub fn verify<P: AsRef<Path>>(path: P, digest: &OciDigest) -> RegistryResult<bool>
{
    let mut buf = Vec::new();
    File::open(path.as_ref())?.read_to_end(&mut buf)?;

    let hash = match digest.get_algo() {
        "sha256" => Sha256::digest(buf).to_vec(),
        "sha512" => Sha512::digest(buf).to_vec(),
        a => err!("Wrong hash algorithm: {}", a)?,
    };

    if hex::encode(&hash) == digest.get_hash() {
        Ok(true)
    } else {
        Ok(false)
    }
}
