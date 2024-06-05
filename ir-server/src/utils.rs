use std::env;
use std::fs::canonicalize;
use std::io::BufReader;
use std::path::Path;
use std::{fs::File, io::Read, io::Write};
use tokio_rustls::rustls::{Certificate, PrivateKey};

use crate::error::RegistryError;
use crate::RegistryResult;

pub fn get_crate_root() -> String
{
    // try with env variable first, it won't work without cargo run
    if let Ok(workspace) = env::var("CARGO_MANIFEST_DIR") {
        return workspace;
    }

    // tentative, assumes the binary is in target/{build}/ directory
    let argv0 = env::args().into_iter().next().unwrap();
    // unwrap() should be safe as argv0 should exist
    let mut workspace = canonicalize(Path::new(&argv0)).unwrap();
    for _ in 1..=3 {
        workspace.pop();
    }

    return workspace.to_string_lossy().to_string();
}

pub fn file_read(filename: &str) -> std::io::Result<Vec<u8>>
{
    let mut buf = Vec::new();
    File::open(filename)?.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn file_write(filename: &str, data: &[u8]) -> std::io::Result<()>
{
    File::create(filename)?.write_all(data)
}

pub(crate) fn load_certificates_from_pem(path: &str) -> std::io::Result<Vec<Certificate>>
{
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let certs = rustls_pemfile::certs(&mut reader)?;

    Ok(certs.into_iter().map(Certificate).collect())
}

pub(crate) fn load_private_key_from_file(path: &str) -> RegistryResult<PrivateKey>
{
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut keys = rustls_pemfile::pkcs8_private_keys(&mut reader)?;

    match keys.len() {
        0 => Err(RegistryError::PrivateKeyParsingError(format!(
            "No PKCS8-encoded private key found in {path}"
        ))),
        1 => Ok(PrivateKey(keys.remove(0))),
        _ => Err(RegistryError::PrivateKeyParsingError(format!(
            "More than one PKCS8-encoded private key found in {path}"
        ))),
    }
}
