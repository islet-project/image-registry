use async_compression::tokio::bufread::{GzipDecoder, ZstdDecoder};
use clean_path::Clean;
use log::{debug, error, warn};
pub use oci_spec::image::MediaType;
use std::path::{Path, PathBuf};
use tokio::{
    fs::{read_dir, remove_dir_all, remove_file, try_exists, File},
    io::{AsyncRead, AsyncReadExt, BufReader},
};
use tokio_stream::*;
use tokio_tar::{Archive, ArchiveBuilder};

use crate::{error::Error, hasher::Hasher, oci::reference::Digest};

const WHITEOUT_FILE: &str = ".wh.";
const WHITEOUT_OPAQUE: &str = ".wh..wh..opq";

enum Whiteout {
    WhiteoutOpaque,
    WhiteoutFile(String),
}

impl Whiteout {
    pub const fn opaque() -> &'static str {
        WHITEOUT_OPAQUE
    }

    pub const fn file() -> &'static str {
        WHITEOUT_FILE
    }
}

pub struct Image {
    root: PathBuf,
}

impl Image {
    pub fn init(dest: impl AsRef<Path>) -> Self {
        Self { root: dest.as_ref().to_path_buf() }
    }

    async fn remove_file(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let orig_file_name = self.root.join(path).clean();
        if !try_exists(&orig_file_name).await? {
            error!("Whiteout points to non-existent file: {}", orig_file_name.display());
            return Err(Error::LayerInvalidError);
        }

        debug!("Removing file {}", orig_file_name.display());
        remove_file(orig_file_name).await?;

        Ok(())
    }

    async fn remove_dir_content(&self, dir: impl AsRef<Path>) -> Result<(), Error> {
        let orig_dir = self.root.join(dir).clean();
        if !try_exists(&orig_dir).await? {
            error!("Whiteout opaque points to non-existent directory");
            return Err(Error::LayerInvalidError);
        }

        let mut read_dir = read_dir(&orig_dir).await?;

        debug!("Removing contents of directory: {}", orig_dir.display());

        while let Some(dir_entry) = read_dir.next_entry().await? {
            if dir_entry.file_type().await?.is_dir() {
                remove_dir_all(dir_entry.path()).await?;
            } else {
                remove_file(dir_entry.path()).await?;
            }
        }

        Ok(())
    }

    fn matches_whiteout(entry: &Path) -> Result<Option<Whiteout>, Error> {
        let Some(file_name) = entry.file_name() else {
            // This shouldn't really happen
            warn!("No file name for regular file");
            return Ok(None);
        };

        let file_name_str = file_name.to_str().ok_or_else ( || {
            error!("Archive entry file name has malformed encoding");
            Error::LayerInvalidError
        })?;

        if file_name_str == Whiteout::opaque() {
            return Ok(Some(Whiteout::WhiteoutOpaque))
        }

        if file_name_str.starts_with(Whiteout::file()) {
            return Ok(Some(Whiteout::WhiteoutFile(file_name_str.to_string())));
        }

        Ok(None)
    }

    async fn process_whiteout<R: AsyncRead + Unpin>(&self, archive: &mut Archive<R>) ->Result<(), Error> {
        debug!("Processing whiteouts");
        let mut entries = archive.entries()?;
        while let Some(entry) = entries.next().await {
            let entry = entry.map_err(|e| {
                error!("Failed to read entries from archive: {e}");
                Error::LayerInvalidError
            })?;

            if !entry.header().entry_type().is_file() {
                // Only a regular file can be a whiteout
                continue
            }

            let entry_path = entry.path().map_err(|e| {
                error!("Archive entry has invalid path: {e}");
                Error::LayerInvalidError
            })?;

            let Some(whiteout) = Self::matches_whiteout(&entry_path)? else {
                // Not a whiteout, ignore
                continue
            };

            debug!("Entry: \"{}\" is a whiteout file", entry_path.display());
            match whiteout {
                Whiteout::WhiteoutOpaque => {
                    self.remove_dir_content(entry_path.parent().unwrap_or(Path::new(""))).await?;
                },
                Whiteout::WhiteoutFile(file_name) => {
                    let orig_file_name = &file_name[Whiteout::file().len()..];
                    let orig_path = entry_path.parent().unwrap_or(Path::new("")).join(orig_file_name).clean();
                    self.remove_file(orig_path).await?;
                },
            }
        }

        Ok(())
    }

    async fn process_copy(&self, reader: impl AsyncRead + Unpin) -> Result<(), Error> {
        let mut archive = ArchiveBuilder::new(reader)
            .set_preserve_permissions(true)
            .set_unpack_xattrs(true)
            .build();

        let mut entries = archive.entries()?;
        while let Some(entry) = entries.next().await {
            let mut entry = entry.map_err(|e| {
                error!("Failed to read entries from archive: {e}");
                Error::LayerInvalidError
            })?;

            let entry_path = entry.path().map_err(|e| {
                error!("Archive entry has invalid path: {e}");
                Error::LayerInvalidError
            })?;

            if entry.header().entry_type().is_file() && Self::matches_whiteout(&entry_path)?.is_some() {
                // We do not unpack whiteout files
                continue;
            };

            let orig_path = self.root.join(&entry_path).clean();

            debug!("Unpacking entry: {} onto: {}", entry_path.display(), orig_path.display());

            entry.unpack_in(&self.root).await?;
        }

        Ok(())
    }

    async fn validate_diff_id<R: AsyncRead + Unpin>(mut hasher: Hasher<R>, diff_id: Digest) -> Result<(), Error> {
        debug!("Validating diff_id: {}", diff_id.value());

        let encoded_diff_id = hex::encode(hasher.finalize());
        if diff_id.value() !=  encoded_diff_id {
            error!("Diff id does not match! Expected: \"{}\", Got: \"{}\"", diff_id.value(), encoded_diff_id);
            return Err(Error::LayerInvalidDiffIdError);
        }
        Ok(())
    }

    pub async fn unpack_layer<P: AsRef<Path>>(
        &self,
        layer: &P,
        media_type: &MediaType,
        diff_id: Digest,
    ) -> Result<(), Error> {
        debug!("Unpacking layer: {} onto directory: {}", layer.as_ref().display(), self.root.display());

        let hash_reader = Hasher::new(diff_id. hash_type(), get_layer_reader(File::open(layer).await?, &media_type)?);

        let mut archive = Archive::new(hash_reader);

        self.process_whiteout(&mut archive).await?;

        // make archive read file to the end, so we can properly compute hash of the file
        loop {
            let mut buf = Vec::with_capacity(1024);
            if archive.read_buf(&mut buf).await? == 0 {
                break
            }
        }

        Self::validate_diff_id(archive.into_inner().map_err(|_| Error::UnknownError)?, diff_id).await?;

        self.process_copy(get_layer_reader(File::open(&layer).await?, media_type)?).await?;

        Ok(())
    }
}

pub fn get_layer_reader<A: AsyncRead + Send + Sync + Unpin + 'static>(
    f: A,
    media_type: &MediaType,
) -> Result<Box<dyn AsyncRead + Send + Sync + Unpin>, Error> {
    match media_type {
        MediaType::ImageLayer => Ok(Box::new(BufReader::new(f))),
        MediaType::ImageLayerGzip => Ok(Box::new(GzipDecoder::new(BufReader::new(f)))),
        MediaType::ImageLayerZstd => Ok(Box::new(ZstdDecoder::new(BufReader::new(f)))),
        _ => {
            error!("Unsupported layer media type: {}", media_type);
            Err(Error::LayerInvalidError)
        },
    }
}
