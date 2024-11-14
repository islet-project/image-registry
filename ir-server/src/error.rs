#[derive(Debug)]
pub enum RegistryError
{
    IO(std::io::Error),
    Rustls(tokio_rustls::rustls::Error),
    SerdeJson(serde_json::Error),
    OciSpec(oci_spec::OciSpecError),
    VeraisonToken(veraison_verifier::VeraisonTokenVeriferError),
    PrivateKeyParsing(String),
    Config(String),
    OciRegistry(String),
    Generic(String),
}

impl std::error::Error for RegistryError {}

impl std::fmt::Display for RegistryError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            RegistryError::IO(e) => write!(f, "IOError({:?})", e),
            RegistryError::Rustls(e) => write!(f, "RustlsError({:?})", e),
            RegistryError::SerdeJson(e) => write!(f, "SerdeJsonError({:?})", e),
            RegistryError::OciSpec(e) => write!(f, "OciSpecError({:?}", e),
            RegistryError::VeraisonToken(e) => write!(f, "VeraisonToken({:?})", e),
            RegistryError::PrivateKeyParsing(s) => write!(f, "PrivateKeyParsingError({})", s),
            RegistryError::Config(s) => write!(f, "ConfigError({})", s),
            RegistryError::OciRegistry(s) => write!(f, "OciRegistryError({})", s),
            RegistryError::Generic(s) => write!(f, "GenericError({})", s),
        }

        // Alternatively:
        //std::fmt::Debug::fmt(&self, f)
    }
}

impl From<std::io::Error> for RegistryError
{
    fn from(value: std::io::Error) -> Self
    {
        Self::IO(value)
    }
}

impl From<tokio_rustls::rustls::Error> for RegistryError
{
    fn from(value: tokio_rustls::rustls::Error) -> Self
    {
        Self::Rustls(value)
    }
}

impl From<serde_json::Error> for RegistryError
{
    fn from(value: serde_json::Error) -> Self
    {
        Self::SerdeJson(value)
    }
}

impl From<oci_spec::OciSpecError> for RegistryError
{
    fn from(value: oci_spec::OciSpecError) -> Self
    {
        Self::OciSpec(value)
    }
}

impl From<veraison_verifier::VeraisonTokenVeriferError> for RegistryError
{
    fn from(value: veraison_verifier::VeraisonTokenVeriferError) -> Self
    {
        Self::VeraisonToken(value)
    }
}

impl From<&'static str> for RegistryError
{
    fn from(value: &'static str) -> Self
    {
        RegistryError::Generic(value.to_string())
    }
}
