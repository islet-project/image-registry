use log::info;
use std::fs::File;
use std::path::Path;

use crate::{crypto, error::SignerError, oci, utils, SignerResult};

const INDEX_JSON: &str = "index.json";
const BLOBS_SUBDIR: &str = "blobs";

macro_rules! err {
    ($($arg:tt)+) => (Err(SignerError::OciRegistry(format!($($arg)+))))
}

pub(crate) fn cmd_generate_key(output: &str) -> SignerResult<()>
{
    info!("Generating key: \"{}\"", output);

    let key = crypto::generate_key();
    let key_u8 = crypto::export_private(&key)?;
    utils::file_write(output, &key_u8)?;

    info!("Key generated");

    Ok(())
}

pub(crate) fn cmd_extract_public(input: &str, output: &str) -> SignerResult<()>
{
    info!("Extracting public key from \"{}\" to \"{}\"", input, output);

    let private_u8 = utils::file_read(input)?;
    let private = crypto::import_private(&private_u8)?;
    let public = crypto::extract_public(&private);
    let public_u8 = crypto::export_public(&public)?;
    utils::file_write(output, &public_u8)?;

    info!("Public key extracted");

    Ok(())
}

#[allow(dead_code)]
pub(crate) fn cmd_sign(key: &str, file: &str, signature: &str) -> SignerResult<()>
{
    info!(
        "Signing file \"{}\" with key \"{}\" into signature \"{}\"",
        file, key, signature
    );

    let der = utils::file_read(key)?;
    let private = crypto::import_private(&der)?;
    let msg = utils::file_read(file)?;

    let sign = crypto::sign(&private, &msg)?;
    utils::file_write(signature, &sign)?;

    info!("File signed, signature written");

    Ok(())
}

pub(crate) fn cmd_sign_buf(key: &str, file: &str, signature: &str) -> SignerResult<()>
{
    info!(
        "Signing file \"{}\" with key \"{}\" into signature \"{}\"",
        file, key, signature
    );

    let der = utils::file_read(key)?;
    let private = crypto::import_private(&der)?;
    let mut msg = File::open(file)?;

    let sign = crypto::sign_reader(&private, &mut msg)?;
    utils::file_write(signature, &sign)?;

    info!("File signed, signature written");

    Ok(())
}

#[allow(dead_code)]
pub(crate) fn cmd_verify(key: &str, file: &str, signature: &str) -> SignerResult<()>
{
    info!(
        "Verifying file \"{}\" with key \"{}\" from signature \"{}\"",
        file, key, signature
    );

    let der = utils::file_read(key)?;
    let public = crypto::import_public(&der)?;
    let msg = utils::file_read(file)?;
    let sign = utils::file_read(signature)?;

    crypto::verify(&public, &msg, &sign).or(err!("Signature verification failed"))?;

    info!("File verified");

    Ok(())
}

pub(crate) fn cmd_verify_buf(key: &str, file: &str, signature: &str) -> SignerResult<()>
{
    info!(
        "Verifying file \"{}\" with key \"{}\" from signature \"{}\"",
        file, key, signature
    );

    let der = utils::file_read(key)?;
    let public = crypto::import_public(&der)?;
    let mut msg = File::open(file)?;
    let sign = utils::file_read(signature)?;

    crypto::verify_reader(&public, &mut msg, &sign).or(err!("Signature verification failed"))?;

    info!("File verified");

    Ok(())
}

pub(crate) fn cmd_sign_config(
    registry: &str,
    app: &str,
    digest: &str,
    vendor_prv: &str,
    vendor_pub_signature: &str,
    ca_pub: &str,
) -> SignerResult<()>
{
    let blobs = Path::new(registry).join(app).join(BLOBS_SUBDIR);
    info!(
        "Signing config in manifest: \"{}\" in: \"{}\"",
        digest,
        blobs.display()
    );

    let vendor_prv = utils::file_read(vendor_prv)?;
    let ca_pub = utils::file_read(ca_pub)?;
    let vendor_sign = utils::file_read(vendor_pub_signature)?;

    oci::verify_vendor_pub_signature(&vendor_prv, &vendor_sign, &ca_pub)?;
    oci::sign_config(&blobs, digest, &vendor_prv, &vendor_sign)?;

    info!("Config signed");

    Ok(())
}

pub(crate) fn cmd_rehash_file(registry: &str, app: &str, digest: &str) -> SignerResult<()>
{
    let blobs = Path::new(registry).join(app).join(BLOBS_SUBDIR);
    info!("Rehashing file: \"{}\" in: \"{}\"", digest, blobs.display());

    let new_digest = oci::rehash_file(&blobs, digest)?;

    if let Some(new_digest) = new_digest {
        info!("Rehashed to: \"{}\"", new_digest);
    } else {
        info!("File does not require renaming");
    }

    Ok(())
}

pub(crate) fn cmd_sign_image(
    registry: &str,
    app: &str,
    digest: &str,
    vendor_prv: &str,
    vendor_pub_signature: Option<&str>,
    ca_pub: Option<&str>,
    ca_prv: Option<&str>,
) -> SignerResult<()>
{
    let vendor_prv = utils::file_read(vendor_prv)?;

    // realistic or simple variant handling
    let vendor_sign = match (vendor_pub_signature, ca_pub, ca_prv) {
        (Some(vendor_pub_signature), Some(ca_pub), None) => {
            let ca_pub = utils::file_read(ca_pub)?;
            let vendor_sign = utils::file_read(vendor_pub_signature)?;
            oci::verify_vendor_pub_signature(&vendor_prv, &vendor_sign, &ca_pub)?;
            vendor_sign
        }
        (None, None, Some(ca_prv)) => {
            let ca_prv = utils::file_read(ca_prv)?;
            oci::sign_vendor_pub(&vendor_prv, &ca_prv)?
        }
        _ => err!("You need to pass either VENDOR_PUB_SIGNATURE and CA_PUB or CA_PRV")?,
    };

    let blobs = Path::new(registry).join(app).join(BLOBS_SUBDIR);
    info!(
        "Signing config for manifest: \"{}\" in: \"{}\"",
        digest,
        blobs.display()
    );
    oci::sign_config(&blobs, digest, &vendor_prv, &vendor_sign)?;

    info!("Rehashing file: \"{}\" in: \"{}\"", digest, blobs.display());
    let new_digest = oci::rehash_file(&blobs, digest)?;

    if let Some(new_digest) = new_digest {
        info!("Rehashed to: \"{}\"", new_digest);
        info!("Updating layout index");
        let index = Path::new("..").join(INDEX_JSON);
        oci::replace_hash_index(&blobs, &index, digest, &new_digest)?;
    } else {
        info!("File does not require renaming");
    }

    info!("Config signed for a given manifest");

    Ok(())
}

pub(crate) fn cmd_extract_sign_image(
    registry: &str,
    filename: &str,
    app: Option<&str>,
    digest: &str,
    vendor_prv: &str,
    vendor_pub_signature: Option<&str>,
    ca_pub: Option<&str>,
    ca_prv: Option<&str>,
) -> SignerResult<()>
{
    let path = Path::new(filename);
    let app_name = match app {
        Some(a) => a,
        None => path.file_stem().unwrap().to_str().unwrap(),
    };

    let app_dir = Path::new(registry).join(app_name);
    info!("Unpacking \"{}\" into \"{}\"", filename, app_dir.display());
    std::fs::create_dir(&app_dir)?;

    let mut tar = tar::Archive::new(File::open(&path)?);
    tar.unpack(&app_dir)?;

    cmd_sign_image(
        registry,
        app_name,
        digest,
        vendor_prv,
        vendor_pub_signature,
        ca_pub,
        ca_prv,
    )
}

pub(crate) fn cmd_verify_image(
    registry: &str,
    app: &str,
    digest: &str,
    ca_pub: &str,
) -> SignerResult<()>
{
    let blobs = Path::new(registry).join(app).join(BLOBS_SUBDIR);
    info!(
        "Verifying config for manifest: \"{}\" in: \"{}\"",
        digest,
        blobs.display()
    );
    let ca_pub = utils::file_read(ca_pub)?;
    oci::verify_config(&blobs, digest, &ca_pub)?;

    info!("Verification succesful");

    Ok(())
}
