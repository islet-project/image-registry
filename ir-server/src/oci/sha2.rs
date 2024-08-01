use sha2::{Digest, Sha256, Sha512};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use super::digest::Digest as OciDigest;
use crate::error::RegistryError;
use crate::RegistryResult;

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistry(format!($($arg)+))))
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

    // unwrap() below is intentional, new() for digest will make sure the hash
    // is correct, verifying digest created with new_unchecked() is an error
    if hash == hex::decode(digest.get_hash()).unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}
