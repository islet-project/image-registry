use oci_spec::image::{ImageIndex, ImageManifest};
use std::collections::HashMap;
use std::path::Path;

use super::crypto;
use super::digest::Digest;
use super::SignerResult;

const ANNOTATION_SIGNATURE: &str = "com.samsung.islet.image.signature";
const ANNOTATION_VENDORPUB: &str = "com.samsung.islet.image.vendorpub";
const ANNOTATION_VENDORPUB_SIGNATURE: &str = "com.samsung.islet.image.vendorpub.signature";

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
    crypto::verify(&c_pub, &v_pub_u8, vendor_pub_signature)?;

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

pub(crate) fn sign_config(
    blobs: &str,
    digest: &str,
    vendor_prv: &[u8],
    vendor_pub_signature: &[u8],
) -> SignerResult<()>
{
    let v_prv = crypto::import_private(vendor_prv)?;
    let v_pub = crypto::extract_public(&v_prv);
    let v_pub_u8 = crypto::export_public(&v_pub)?;

    let blobs = Path::new(blobs);

    // load manifest
    let manifest_digest = Digest::try_from(digest)?;
    let manifest_path = blobs.join(manifest_digest.to_path());
    let mut manifest = ImageManifest::from_file(&manifest_path)?;

    // find config
    let config_desc = manifest.config();
    let config_digest = Digest::try_from(config_desc.digest())?;
    let config_path = blobs.join(config_digest.to_path());

    // sign the config with vendor key
    let mut config = std::fs::File::open(config_path)?;
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

pub(crate) fn rehash_file(blobs: &str, digest: &str) -> SignerResult<Option<String>>
{
    let blobs = Path::new(blobs);
    let digest = Digest::try_from(digest)?;
    let path = blobs.join(digest.to_path());
    let mut file = std::fs::File::open(&path)?;

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

pub(crate) fn replace_hash_index(blobs: &str, file: &str, from: &str, to: &str)
    -> SignerResult<()>
{
    let blobs = Path::new(blobs);
    let path = blobs.join(file);

    let mut index = ImageIndex::from_file(&path)?;
    let mut manifests = index.manifests().clone();

    for descriptor in &mut manifests {
        if descriptor.digest() == from {
            let new_digest = Digest::try_from(to)?;
            let path = blobs.join(new_digest.to_path());
            let size = std::fs::metadata(&path)?.len();

            descriptor.set_digest(to.to_string());
            descriptor.set_size(size.try_into().unwrap());
        }
    }

    index.set_manifests(manifests);
    index.to_file_pretty(&path)?;

    Ok(())
}
