#[derive(Debug)]
pub enum RegistryError
{
    IOError(std::io::Error),
    RustlsError(tokio_rustls::rustls::Error),
    SerdeYamlError(serde_yaml::Error),
    SerdeJsonError(serde_json::Error),
    PrivateKeyParsingError(String),
}

impl std::error::Error for RegistryError {}

impl std::fmt::Display for RegistryError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        write!(f, "RaTlsError")
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
