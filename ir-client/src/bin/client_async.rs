use ir_client::async_client::Client;

use clap::{Args, Parser, Subcommand};
use log::info;
use tokio::io::AsyncReadExt;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    GetManifest(GetManifestArgs),
    GetImage(GetImageArgs),
}

#[derive(Args, Debug)]
struct GetManifestArgs {
    /// Host url address
    #[arg(short = 'a', long, default_value = "http://localhost:1337")]
    host: String,

    /// Uuid of image
    #[arg(short, long)]
    uuid: String,

    /// filename to write image JSON [default: ./{uuid}.json]
    #[arg(short, long)]
    out: Option<String>,
}

#[derive(Args, Debug)]
struct GetImageArgs {
    // Host url address
    #[arg(short = 'a', long, default_value = "http://localhost:1337")]
    host: String,

    // Uuid of image
    #[arg(short, long)]
    uuid: String,

    /// filename to write image archive [default: ./{uuid}.tar.gz]
    #[arg(short, long)]
    out: Option<String>,
}

#[tokio::main]
async fn main() -> () {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    println!("{:?}", cli.command);
    match &cli.command {
        Commands::GetManifest(args) => {
            let manifest = Client::new(args.host.clone())
                .get_manifest(uuid::Uuid::parse_str(&args.uuid).unwrap())
                .await
                .unwrap();
            info!("Manifest: {:?}", manifest);
        }
        Commands::GetImage(args) => {
            let mut image_bytes = Client::new(args.host.clone())
                .get_image_stream(uuid::Uuid::parse_str(&args.uuid).unwrap())
                .await
                .unwrap();

            let mut buf = Vec::new();
            image_bytes.read_to_end(&mut buf).await.unwrap();
            info!("Image size {}", buf.len());
        }
    }
}
