use std::env;
use std::fs::canonicalize;
use std::io::BufReader;
use std::path::Path;
use std::{fs::File, io::Read, io::Write};
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};

use crate::error::RegistryError;
use crate::RegistryResult;

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::PrivateKeyParsing(format!($($arg)+))))
}

pub fn get_crate_root() -> String
{
    // try with env variable first, it won't work without cargo run
    if let Ok(workspace) = env::var("CARGO_MANIFEST_DIR") {
        return workspace;
    }

    // tentative, assumes the binary is in target/{build}/ directory
    let argv0 = env::args().next().unwrap();
    // unwrap() should be safe as argv0 should exist
    let mut workspace = canonicalize(Path::new(&argv0)).unwrap();
    for _ in 1..=3 {
        workspace.pop();
    }

    return workspace.to_string_lossy().to_string();
}

#[allow(dead_code)]
pub fn file_read<T: AsRef<Path>>(filename: T) -> std::io::Result<Vec<u8>>
{
    let mut buf = Vec::new();
    File::open(filename)?.read_to_end(&mut buf)?;
    Ok(buf)
}

#[allow(dead_code)]
pub fn file_write<T: AsRef<Path>>(filename: T, data: &[u8]) -> std::io::Result<()>
{
    File::create(filename)?.write_all(data)
}

#[allow(dead_code)]
pub fn file_len<T: AsRef<Path>>(filename: T) -> std::io::Result<u64>
{
    Ok(std::fs::metadata(filename)?.len())
}

pub(crate) fn load_certificates_from_pem<'a, T: AsRef<Path>>(
    path: T,
) -> std::io::Result<Vec<CertificateDer<'a>>>
{
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    rustls_pemfile::certs(&mut reader).collect()
}

pub(crate) fn load_private_key_from_file<'a, T>(path: T) -> RegistryResult<PrivateKeyDer<'a>>
where
    T: AsRef<Path> + std::fmt::Display,
{
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut keys =
        rustls_pemfile::pkcs8_private_keys(&mut reader).collect::<Result<Vec<_>, _>>()?;
    match keys.len() {
        0 => err!("No PKCS8-encoded private key found in {path}"),
        1 => Ok(PrivateKeyDer::Pkcs8(keys.remove(0))),
        _ => err!("More than one PKCS8-encoded private key found in {path}"),
    }
}
