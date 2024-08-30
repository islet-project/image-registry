use std::fmt::Display;

use crate::{error::{self, Error}, hasher::HashType};
use log::{error, info};
use regex::Regex;

pub(crate) const SHA_256: &str = "sha256";
pub(crate) const SHA_512: &str = "sha512";

#[derive(Debug, Clone)]
pub struct Digest {
    hash_type: HashType,
    value: String,
}

impl Digest {
    const REGEX: &'static str = r"^([a-z0-9]+[[+._-][a-z0-9]+]*):([a-zA-Z0-9=_-]+)$";

    pub fn hash_type(&self) -> &HashType {
        &self.hash_type
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    fn from_str(value: &str) -> Option<Self> {
        let digest_re = Regex::new(Self::REGEX).expect("Digest regex is malformed");
        let Some(captures) = digest_re.captures(value) else {
            info!("Reference not a digest");
            return None;
        };

        let (_, [algorithm, digest]) = captures.extract();
        match (algorithm, digest.len()) {
            (SHA_256, 64) => Some(
                Digest { hash_type: HashType::Sha256, value: digest.to_string() }
            ),
            (SHA_256, _) => {
                error!("Wrong length for sha256: {}", digest.len());
                None
            },

            (SHA_512, 128) => Some(
                Digest { hash_type: HashType::Sha512, value: digest.to_string() }
            ),
            (SHA_512, _) => {
                error!("Wrong length for sha512: {}", digest.len());
                None
            },
            (a, _) => {
                error!("Unrecognized digest algorithm: {}", a);
                None
            }
        }
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.hash_type, self.value)
    }
}

impl TryFrom<&str> for Digest {
    type Error = error::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value).ok_or(Error::DigestInvalidError)
    }
}

#[derive(Debug, Clone)]
pub struct Tag(String);

impl Tag {
    const REGEX: &'static str = r"^[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}$";

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn from_str(value: &str) -> Option<Self> {
        let tag_re = Regex::new(Self::REGEX).expect("Tag regex is malformed");
        match tag_re.is_match(value) {
            true => Some(Tag(value.to_string())),
            false => {
                info!("Reference not a tag");
                None
            },
        }
    }
}

impl TryFrom<&str> for Tag {
    type Error = error::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value).ok_or(Error::TagInvalidError)
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub enum Reference {
    Digest(Digest),
    Tag(Tag),
}

impl TryFrom<&str> for Reference {
    type Error = error::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Ok(digest) = Digest::try_from(value) {
            return Ok(Self::Digest(digest));
        }

        if let Ok(tag) = Tag::try_from(value) {
            return Ok(Self::Tag(tag));
        }

        error!("Reference is not a digest nor a tag");
        Err(Error::ReferenceInvalidError)
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Digest(digest) => write!(f, "{}", digest),
            Self::Tag(tag) => write!(f, "{}", tag),
        }
    }
}
