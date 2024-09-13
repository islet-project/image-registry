use oci_spec::image::{ImageIndex, ImageManifest};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::{crypto, digest::Digest, error::SignerError, utils, SignerResult};

const ANNOTATION_SIGNATURE: &str = "com.samsung.islet.image.signature";
const ANNOTATION_VENDORPUB: &str = "com.samsung.islet.image.vendorpub";
const ANNOTATION_VENDORPUB_SIGNATURE: &str = "com.samsung.islet.image.vendorpub.signature";

macro_rules! err {
    ($($arg:tt)+) => (Err(SignerError::OciRegistry(format!($($arg)+))))
}
macro_rules! er {
    ($($arg:tt)+) => (SignerError::OciRegistry(format!($($arg)+)))
}

pub(crate) fn verify_vendor_pub_signature(
    vendor_prv: &[u8],
    vendor_pub_signature: &[u8],
    ca_pub: &[u8],
) -> SignerResult<()>
{
    let v_prv = crypto::import_private(vendor_prv)?;
    let v_pub = crypto::extract_public(&v_prv);
    let v_pub_u8 = crypto::export_public(&v_pub)?;
    let c_pub = crypto::import_public(ca_pub)?;

    // verify the vendor_pub_signature with public ca key
    crypto::verify(&c_pub, &v_pub_u8, vendor_pub_signature)
        .or(err!("Vendor pub signature verification failed"))?;

    Ok(())
}

pub(crate) fn sign_vendor_pub(vendor_prv: &[u8], ca_prv: &[u8]) -> SignerResult<Vec<u8>>
{
    let v_prv = crypto::import_private(vendor_prv)?;
    let v_pub = crypto::extract_public(&v_prv);
    let v_pub_u8 = crypto::export_public(&v_pub)?;
    let c_prv = crypto::import_private(ca_prv)?;

    // sign the vendor verification/public key with private ca key
    let v_sign = crypto::sign(&c_prv, &v_pub_u8)?;

    Ok(v_sign)
}

pub(crate) fn sign_config<T: AsRef<Path>>(
    blobs: T,
    digest: &str,
    vendor_prv: &[u8],
    vendor_pub_signature: &[u8],
) -> SignerResult<()>
{
    let blobs = blobs.as_ref();

    let v_prv = crypto::import_private(vendor_prv)?;
    let v_pub = crypto::extract_public(&v_prv);
    let v_pub_u8 = crypto::export_public(&v_pub)?;

    // load manifest
    let manifest_digest = Digest::try_from(digest)?;
    let manifest_path = blobs.join(manifest_digest.to_path());
    let mut manifest = ImageManifest::from_file(&manifest_path)?;

    // find config
    let config_desc = manifest.config();
    let config_digest = Digest::try_from(config_desc.digest())?;
    let config_path = blobs.join(config_digest.to_path());

    // sign the config with vendor key
    let mut config = File::open(config_path)?;
    let config_sign = crypto::sign_reader(&v_prv, &mut config)?;

    // get/create annotations
    let annotations = manifest.annotations_mut();
    let annotations = match annotations {
        None => {
            let hm = HashMap::<String, String>::new();
            manifest.set_annotations(Some(hm));
            manifest.annotations_mut().as_mut().unwrap()
        }
        Some(a) => a,
    };

    // ammend annotations
    annotations.insert(ANNOTATION_SIGNATURE.to_string(), hex::encode(config_sign));
    annotations.insert(ANNOTATION_VENDORPUB.to_string(), hex::encode(v_pub_u8));
    annotations.insert(
        ANNOTATION_VENDORPUB_SIGNATURE.to_string(),
        hex::encode(vendor_pub_signature),
    );

    manifest.to_file_pretty(&manifest_path)?;

    Ok(())
}

pub(crate) fn rehash_file<T: AsRef<Path>>(blobs: T, digest: &str) -> SignerResult<Option<String>>
{
    let blobs = blobs.as_ref();
    let digest = Digest::try_from(digest)?;
    let path = blobs.join(digest.to_path());
    let mut file = File::open(&path)?;

    let hash = crypto::hash_reader(digest.algo(), &mut file)?;

    let new_digest = Digest::new_unchecked(digest.algo().to_string(), hex::encode(hash));

    if digest == new_digest {
        Ok(None)
    } else {
        let new_path = blobs.join(new_digest.to_path());
        std::fs::rename(&path, &new_path)?;
        Ok(Some(new_digest.into()))
    }
}

pub(crate) fn replace_hash_index<T: AsRef<Path>>(
    blobs: T,
    file: T,
    from: &str,
    to: &str,
) -> SignerResult<()>
{
    let blobs = blobs.as_ref();
    let path = blobs.join(file);

    let mut index = ImageIndex::from_file(&path)?;
    let mut manifests = index.manifests().clone();

    for descriptor in &mut manifests {
        if descriptor.digest() == from {
            let new_digest = Digest::try_from(to)?;
            let path = blobs.join(new_digest.to_path());
            let size = utils::file_len(&path)?;

            descriptor.set_digest(to.to_string());
            descriptor.set_size(size.try_into().unwrap());
        }
    }

    index.set_manifests(manifests);
    index.to_file_pretty(&path)?;

    Ok(())
}

pub(crate) fn verify_config<T: AsRef<Path>>(
    blobs: T,
    digest: &str,
    ca_pub: &[u8],
) -> SignerResult<()>
{
    let blobs = blobs.as_ref();

    // load manifest
    let manifest_digest = Digest::try_from(digest)?;
    let manifest_path = blobs.join(manifest_digest.to_path());
    let manifest = ImageManifest::from_file(&manifest_path)?;

    // find config
    let config_desc = manifest.config();
    let config_digest = Digest::try_from(config_desc.digest())?;
    let config_path = blobs.join(config_digest.to_path());

    // check config hash first, just to be sure
    let mut config = File::open(&config_path)?;
    let config_hash = crypto::hash_reader(config_digest.algo(), &mut config)?;
    // config_hash is raw binary [u8], config_digest is hex String
    let config_digest_hash = hex::decode(config_digest.hash().as_bytes())?;
    if config_hash != config_digest_hash {
        err!("Config hash mismatch")?;
    }

    // read data from manifest annotations
    let annotations = manifest.annotations();
    let Some(annotations) = annotations else {
        return err!("Manifest does not contain annotations");
    };
    let config_sign = annotations
        .get(ANNOTATION_SIGNATURE)
        .ok_or(er!("Missing SIGNATURE annotation"))?;
    let v_pub = annotations
        .get(ANNOTATION_VENDORPUB)
        .ok_or(er!("Missing VENDORPUB annotation"))?;
    let v_pub_sig = annotations
        .get(ANNOTATION_VENDORPUB_SIGNATURE)
        .ok_or(er!("Missing VENDORPUB_SIGNATURE annotation"))?;

    // all annotations are hex strings, convert to something usable
    let config_sig = hex::decode(config_sign.as_bytes())?;
    let v_pub_u8 = hex::decode(v_pub.as_bytes())?;
    let v_pub_sig = hex::decode(v_pub_sig.as_bytes())?;

    // verify the vendor_pub key
    let c_pub = crypto::import_public(ca_pub)?;
    crypto::verify(&c_pub, &v_pub_u8, &v_pub_sig)
        .or(err!("Vendor pub signature verification failed"))?;

    // verify the config signature
    let v_pub = crypto::import_public(&v_pub_u8)?;
    let mut config = File::open(&config_path)?;
    crypto::verify_reader(&v_pub, &mut config, &config_sig)
        .or(err!("Config signature verification failed"))?;

    Ok(())
}
