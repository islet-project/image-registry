#[derive(Debug)]
pub enum RegistryError
{
    IOError(std::io::Error),
    RustlsError(tokio_rustls::rustls::Error),
    SerdeYamlError(serde_yaml::Error),
    SerdeJsonError(serde_json::Error),
    PrivateKeyParsingError(String),
    GenericError(String),
}

impl std::error::Error for RegistryError {}

impl std::fmt::Display for RegistryError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            RegistryError::IOError(e) => write!(f, "IOError({:?})", e),
            RegistryError::RustlsError(e) => write!(f, "RustlsError({:?})", e),
            RegistryError::SerdeYamlError(e) => write!(f, "SerdeYamlError({:?})", e),
            RegistryError::SerdeJsonError(e) => write!(f, "SerdeJsonError({:?})", e),
            RegistryError::PrivateKeyParsingError(s) => write!(f, "PrivateKeyParsingError({})", s),
            RegistryError::GenericError(s) => write!(f, "Generic({})", s),
        }

        // Alternatively:
        //std::fmt::Debug::fmt(&self, f)
    }
}

impl From<std::io::Error> for RegistryError
{
    fn from(value: std::io::Error) -> Self
    {
        Self::IOError(value)
    }
}

impl From<tokio_rustls::rustls::Error> for RegistryError
{
    fn from(value: tokio_rustls::rustls::Error) -> Self
    {
        Self::RustlsError(value)
    }
}

impl From<serde_yaml::Error> for RegistryError
{
    fn from(value: serde_yaml::Error) -> Self
    {
        Self::SerdeYamlError(value)
    }
}

impl From<serde_json::Error> for RegistryError
{
    fn from(value: serde_json::Error) -> Self
    {
        Self::SerdeJsonError(value)
    }
}

impl From<&'static str> for RegistryError
{
    fn from(value: &'static str) -> Self
    {
        RegistryError::GenericError(value.to_string())
    }
}
