use regex::Regex;
use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use crate::{error::RegistryError, RegistryResult};

const SHA256_LEN: usize = 64;
const SHA512_LEN: usize = 128;

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistryError(format!($($arg)+))))
}

#[derive(Eq, Hash)]
pub struct Digest
{
    algo: String,
    hash: String,
}

impl PartialEq for Digest
{
    fn eq(&self, other: &Self) -> bool
    {
        self.to_string() == other.to_string()
    }
}

impl Display for Digest
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        <Self as Debug>::fmt(&self, f)
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
        Digest::try_from(&value as &str)
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

impl Into<String> for Digest
{
    fn into(self) -> String
    {
        self.algo + ":" + &self.hash
    }
}

impl Into<PathBuf> for Digest
{
    fn into(self) -> PathBuf
    {
        [self.algo, self.hash].iter().collect()
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

        let re = Regex::new(r"^[a-zA-Z0-9]+$").unwrap();
        if !re.is_match(&hash) {
            err!("Hash doesn't match the pattern: {}", re)?;
        }

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

    pub fn to_string(&self) -> String
    {
        format!("{}:{}", self.algo, self.hash)
    }

    pub fn to_path(&self) -> PathBuf
    {
        [self.algo.clone(), self.hash.clone()].iter().collect()
    }
}
