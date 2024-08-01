use oci_spec::image::{ImageIndex, ImageManifest, MediaType, OciLayout, SCHEMA_VERSION};

use crate::error::RegistryError;
use crate::RegistryResult;

const OCI_LAYOUT_VERSION: &str = "1.0.0";

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::OciRegistry(format!($($arg)+))))
}

pub trait Validate
{
    fn validate(&self) -> RegistryResult<()>;
}

impl Validate for OciLayout
{
    fn validate(&self) -> RegistryResult<()>
    {
        let image_layout_version: &str = self.image_layout_version();
        if image_layout_version != OCI_LAYOUT_VERSION {
            err!(
                "Wrong OCI layout version: {}, expected: {}",
                image_layout_version,
                OCI_LAYOUT_VERSION
            )?;
        }

        Ok(())
    }
}

impl Validate for ImageIndex
{
    fn validate(&self) -> RegistryResult<()>
    {
        let schema_version = self.schema_version();
        if schema_version != SCHEMA_VERSION {
            err!(
                "Wrong index schemaVersion: {:?}, expected: {}",
                schema_version,
                SCHEMA_VERSION
            )?;
        }

        let media_type = self.media_type();
        if media_type != &Some(MediaType::ImageIndex) {
            err!(
                "Wrong index mediaType: {:?}, expected: {}",
                media_type,
                MediaType::ImageIndex
            )?;
        }

        Ok(())
    }
}

impl Validate for ImageManifest
{
    fn validate(&self) -> RegistryResult<()>
    {
        let schema_version = self.schema_version();
        if schema_version != SCHEMA_VERSION {
            err!(
                "Wrong index schemaVersion: {:?}, expected: {}",
                schema_version,
                SCHEMA_VERSION
            )?;
        }

        let media_type = self.media_type();
        if media_type != &Some(MediaType::ImageManifest) {
            err!(
                "Wrong index mediaType: {:?}, expected: {}",
                media_type,
                MediaType::ImageManifest
            )?;
        }

        Ok(())
    }
}
