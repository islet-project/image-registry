use crate::error::{self, Error};
use log::{error, info};
use regex::Regex;

#[derive(Debug)]
pub struct Digest(String);

impl Digest {
    const REGEX: &'static str = r"^([a-z0-9]+[[+._-][a-z0-9]+]*):([a-zA-Z0-9=_-]+)$";

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn from_str(value: &str) -> Option<Self> {
        let digest_re = Regex::new(Self::REGEX).expect("Digest regex is malformed");
        let Some(captures) = digest_re.captures(value) else {
            info!("Reference not a digest");
            return None;
        };

        let (_, [algorithm, digest]) = captures.extract();
        match (algorithm, digest.len()) {
            ("sha256", 64) => Some(Digest(value.to_string())),
            ("sha256", _) => {
                error!("Wrong length for sha256: {}", digest.len());
                None
            },

            ("sha512", 128) => Some(Digest(value.to_string())),
            ("sha512", _) => {
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

impl TryFrom<&str> for Digest {
    type Error = error::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value).ok_or(Error::DigestInvalidError)
    }
}

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

impl Reference {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Digest(digest) => digest.as_str(),
            Self::Tag(tag) => tag.as_str(),
        }
    }
}
