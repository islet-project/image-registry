use clap::Parser;
use ir_client::{layer::Image, oci::reference::Digest};
use oci_spec::image::MediaType;
use serde::Serialize;

#[derive(clap::ValueEnum, Clone, Default, Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ClientMediaType {
    /// tar file
    #[default]
    Tar,
    /// tar file compressed by Gzip
    TarGz,
    /// tar file compressed by Zstd
    TarZstd,
}

impl ClientMediaType {
    pub fn into_media_type(self) -> MediaType {
        match self {
            Self::Tar => MediaType::ImageLayer,
            Self::TarGz => MediaType::ImageLayerGzip,
            Self::TarZstd => MediaType::ImageLayerZstd,
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of directory for the layer to be applied to
    #[arg(short, long, id = "DIR")]
    dest: String,

    /// Path to layer file
    #[arg(short, long, id = "PATH")]
    layer: String,

    /// Type of a layer file
    #[arg(short = 't', long = "type", default_value_t, value_enum)]
    media_type: ClientMediaType,

    /// Diff_id to check
    #[arg(short='i', long)]
    diff_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let args = Args::parse();
    let image = Image::init(args.dest);
    image.unpack_layer(
        args.layer,
        args.media_type.into_media_type(),
        Digest::try_from(args.diff_id.as_str()).unwrap(),
    ).await.unwrap();
    return Ok(());
}
