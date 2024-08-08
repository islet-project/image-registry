use std::{error::Error as StdError, fmt::Display, io::Error as IOError};

#[derive(Debug)]
pub enum Error {
    ConfigError(reqwest::Error),
    UrlParsingError(String),
    JSONParsingError(String),
    ManifestFormatError,

    ConnectionError,
    IOError(IOError),
    StatusError(u16),

    ReferenceInvalidError,
    DigestInvalidError,
    TagInvalidError,

    UnknownError,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigError(reqwest_error) => write!(f, "Configuration error: {}", reqwest_error)?,
            Self::ConnectionError => f.write_str("Connection error")?,
            Self::DigestInvalidError => f.write_str("Invalid digest format")?,
            Self::IOError(io_error) => write!(f, "IO error: {}", io_error)?,
            Self::JSONParsingError(json_error) => write!(f, "JSON parsing error: {}", json_error)?,
            Self::ManifestFormatError => f.write_str("Invalid manifest format")?,
            Self::ReferenceInvalidError => f.write_str("Invalid reference format")?,
            Self::StatusError(status_error) => write!(f, "HTTP status error code: {}", status_error)?,
            Self::TagInvalidError => f.write_str("Invalid tag format")?,
            Self::UnknownError => f.write_str("Unknown error")?,
            Self::UrlParsingError(url_error) => write!(f, "Url parsing error: {}", url_error)?,
        }

        Ok(())
    }
}

impl StdError for Error {}

impl From<url::ParseError> for Error {
    fn from(value: url::ParseError) -> Self {
        Error::UrlParsingError(value.to_string())
    }
}

impl From<IOError> for Error {
    fn from(value: IOError) -> Self {
        Error::IOError(value)
    }
}


impl Error {
    pub fn into_config(reqwest_error: reqwest::Error) -> Self {
        Self::ConfigError(reqwest_error)
    }
}
