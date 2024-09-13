use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use crate::{error::RegistryError, RegistryResult};

const SHA256_LEN: usize = 64;
const SHA512_LEN: usize = 128;

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistry(format!($($arg)+))))
}

macro_rules! er {
    ($($arg:tt)+) => (RegistryError::OciRegistry(format!($($arg)+)))
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Digest
{
    algo: String,
    hash: String,
}

impl Display for Digest
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        <Self as Debug>::fmt(self, f)
    }
}

impl Debug for Digest
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "{}:{}", self.algo, self.hash)
    }
}

impl TryFrom<&String> for Digest
{
    type Error = RegistryError;

    fn try_from(value: &String) -> Result<Self, Self::Error>
    {
        Digest::try_from(value as &str)
    }
}

impl TryFrom<String> for Digest
{
    type Error = RegistryError;

    fn try_from(value: String) -> Result<Self, Self::Error>
    {
        Digest::try_from(&value as &str)
    }
}

impl TryFrom<&str> for Digest
{
    type Error = RegistryError;

    fn try_from(value: &str) -> Result<Self, Self::Error>
    {
        let parts: Vec<_> = value.split(":").collect();
        if parts.len() != 2 {
            err!("Digest should contain exactly 2 parts, not {}", parts.len())?;
        }

        Self::new(parts[0].to_string(), parts[1].to_string())
    }
}

impl From<Digest> for String
{
    fn from(value: Digest) -> Self
    {
        value.algo + ":" + &value.hash
    }
}

impl From<Digest> for PathBuf
{
    fn from(value: Digest) -> Self
    {
        [value.algo, value.hash].iter().collect()
    }
}

impl Digest
{
    pub fn new(algo: String, hash: String) -> RegistryResult<Self>
    {
        let hash_len = hash.len();
        match algo.as_str() {
            "sha256" => {
                if hash_len != SHA256_LEN {
                    err!("Wrong hash length: {}, expected: {}", hash_len, SHA256_LEN)?;
                }
            }
            "sha512" => {
                if hash_len != SHA512_LEN {
                    err!("Wrong hash length: {}, expected: {}", hash_len, SHA512_LEN)?;
                }
            }
            a => err!("Wrong hash algorithm: {}", a)?,
        }

        hex::decode(&hash).map_err(|e| er!("Incorrect hash string: {}", e))?;

        Ok(Digest { algo, hash })
    }

    pub fn new_unchecked(algo: String, hash: String) -> Self
    {
        Digest { algo, hash }
    }

    pub fn get_algo(&self) -> &str
    {
        &self.algo
    }

    pub fn get_hash(&self) -> &str
    {
        &self.hash
    }

    pub fn to_path(&self) -> PathBuf
    {
        [self.algo.clone(), self.hash.clone()].iter().collect()
    }
}
