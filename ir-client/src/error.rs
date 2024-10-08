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

    LayerInvalidError,
    LayerInvalidDiffIdError,

    ResponseLengthInvalid,
    ResponseDigestInvalid,
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
            Self::ResponseLengthInvalid => f.write_str("Response length invalid")?,
            Self::ResponseDigestInvalid => f.write_str("Response digest invalid")?,
            Self::LayerInvalidError => f.write_str("Layer invalid")?,
            Self::LayerInvalidDiffIdError => f.write_str("Layer diff_id invalid")?,
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

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::JSONParsingError(value.to_string())
    }
}

impl Error {
    pub fn into_config(reqwest_error: reqwest::Error) -> Self {
        Self::ConfigError(reqwest_error)
    }
}
