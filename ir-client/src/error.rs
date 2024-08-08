use std::io::Error as IOError;

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
