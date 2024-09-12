#[derive(Debug)]
pub enum SignerError
{
    IO(std::io::Error),
    Sec1(sec1::Error),
    Spki(spki::Error),
    Ecdsa(p384::ecdsa::Error),
    Der(sec1::der::Error),
    OciSpec(oci_spec::OciSpecError),
    Crypto(String),
    OciRegistry(String),
    Generic(String),
}

impl std::error::Error for SignerError {}

impl std::fmt::Display for SignerError
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        match self {
            SignerError::IO(e) => write!(f, "IOError({:?})", e),
            SignerError::Sec1(e) => write!(f, "Sec1Error({:?})", e),
            SignerError::Spki(e) => write!(f, "SpkiError({:?})", e),
            SignerError::Ecdsa(e) => write!(f, "EcdsaError({:?})", e),
            SignerError::Der(e) => write!(f, "DerError({:?})", e),
            SignerError::OciSpec(e) => write!(f, "OciSpecError({:?})", e),
            SignerError::Crypto(s) => write!(f, "CryptoError({})", s),
            SignerError::OciRegistry(s) => write!(f, "OciRegistryError({})", s),
            SignerError::Generic(s) => write!(f, "GenericError({})", s),
        }

        // Alternatively:
        //std::fmt::Debug::fmt(&self, f)
    }
}

impl From<std::io::Error> for SignerError
{
    fn from(value: std::io::Error) -> Self
    {
        Self::IO(value)
    }
}

impl From<sec1::Error> for SignerError
{
    fn from(value: sec1::Error) -> Self
    {
        Self::Sec1(value)
    }
}

impl From<spki::Error> for SignerError
{
    fn from(value: spki::Error) -> Self
    {
        Self::Spki(value)
    }
}

impl From<p384::ecdsa::Error> for SignerError
{
    fn from(value: p384::ecdsa::Error) -> Self
    {
        Self::Ecdsa(value)
    }
}

impl From<sec1::der::Error> for SignerError
{
    fn from(value: sec1::der::Error) -> Self
    {
        Self::Der(value)
    }
}

impl From<oci_spec::OciSpecError> for SignerError
{
    fn from(value: oci_spec::OciSpecError) -> Self
    {
        Self::OciSpec(value)
    }
}

impl From<&'static str> for SignerError
{
    fn from(value: &'static str) -> Self
    {
        SignerError::Generic(value.to_string())
    }
}
