use std::io::Error as IOError;
use url::ParseError;

#[derive(Debug)]
pub enum Error {
    UnknownUuid,
    UrlParsingError(String),
    JSONParsingError(String),
    ManifestFormatError,
    ConnectionError,
    IOError(IOError),
    StatusError(u16),
    UnknownError,
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Error::UrlParsingError(value.to_string())
    }
}

impl From<IOError> for Error {
    fn from(value: IOError) -> Self {
        Error::IOError(value)
    }
}
