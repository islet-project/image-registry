use p384::ecdsa::{
    signature::{DigestSigner, Signer},
    Signature, SigningKey,
};
use p384::ecdsa::{
    signature::{DigestVerifier, Verifier},
    VerifyingKey,
};
use rand_core::OsRng;
use sec1::{DecodeEcPrivateKey, EncodeEcPrivateKey};
use sha2::{digest::DynDigest, Digest, Sha256, Sha384, Sha512};
use spki::{der::Encode, DecodePublicKey, EncodePublicKey};
use std::io;

use super::{error::SignerError, SignerResult};

const READ_BUF_SIZE: usize = 4096;

macro_rules! err {
    ($($arg:tt)+) => (Err(SignerError::Crypto(format!($($arg)+))))
}

pub(crate) fn import_private(input: &[u8]) -> SignerResult<SigningKey>
{
    Ok(SigningKey::from_sec1_der(input)?)
}

pub(crate) fn import_public(input: &[u8]) -> SignerResult<VerifyingKey>
{
    Ok(VerifyingKey::from_public_key_der(input)?)
}

pub(crate) fn export_private(key: &SigningKey) -> SignerResult<Vec<u8>>
{
    let bytes = key.to_sec1_der()?.to_bytes();
    Ok(bytes.to_vec())
}

pub(crate) fn export_public(key: &VerifyingKey) -> SignerResult<Vec<u8>>
{
    let bytes = key.to_public_key_der()?;
    Ok(bytes.to_vec())
}

pub(crate) fn generate_key() -> SigningKey
{
    SigningKey::random(&mut OsRng)
}

pub(crate) fn extract_public(key: &SigningKey) -> VerifyingKey
{
    *key.verifying_key()
}

pub(crate) fn sign(key: &SigningKey, msg: &[u8]) -> SignerResult<Vec<u8>>
{
    let sign: Signature = key.try_sign(msg)?;
    let mut vec = Vec::new();
    sign.to_der().encode_to_vec(&mut vec)?;
    Ok(vec)
}

pub(crate) fn sign_reader<T: io::Read>(key: &SigningKey, reader: &mut T) -> SignerResult<Vec<u8>>
{
    let mut hasher = Sha384::new();
    let mut buf = vec![0u8; READ_BUF_SIZE];
    loop {
        let n = reader.read(buf.as_mut_slice())?;
        if n == 0 {
            break;
        }
        Digest::update(&mut hasher, &buf[0..n]);
    }
    let sign: Signature = key.sign_digest(hasher);
    let mut vec = Vec::new();
    sign.to_der().encode_to_vec(&mut vec)?;
    Ok(vec)
}

pub(crate) fn verify(key: &VerifyingKey, msg: &[u8], signature: &[u8]) -> SignerResult<()>
{
    let sign = Signature::from_der(signature)?;
    key.verify(msg, &sign)?;
    Ok(())
}

pub(crate) fn verify_reader<T: io::Read>(
    key: &VerifyingKey,
    reader: &mut T,
    signature: &[u8],
) -> SignerResult<()>
{
    let sign = Signature::from_der(signature)?;
    let mut hasher = Sha384::new();
    let mut buf = vec![0u8; READ_BUF_SIZE];
    loop {
        let n = reader.read(buf.as_mut_slice())?;
        if n == 0 {
            break;
        }
        Digest::update(&mut hasher, &buf[0..n]);
    }
    key.verify_digest(hasher, &sign)?;
    Ok(())
}

pub(crate) fn hash_reader<T: io::Read>(algo: &str, reader: &mut T) -> SignerResult<Vec<u8>>
{
    let mut buf = vec![0u8; READ_BUF_SIZE];

    let mut hasher: Box<dyn DynDigest> = match algo {
        "sha256" => Box::new(Sha256::new()),
        "sha384" => Box::new(Sha384::new()),
        "sha512" => Box::new(Sha512::new()),
        a => err!("Wrong hash algorithm: {}", a)?,
    };

    loop {
        let n = reader.read(buf.as_mut_slice())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[0..n]);
    }

    Ok(hasher.finalize().to_vec())
}
