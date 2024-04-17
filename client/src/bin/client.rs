use client::Client;

use clap::{Args, Parser, Subcommand};
use log::info;

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
    #[arg(short = 'a', long, default_value = "http://localhost:8888")]
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
    #[arg(short = 'a', long, default_value = "http://localhost:8888")]
    host: String,

    // Uuid of image
    #[arg(short, long)]
    uuid: String,

    /// filename to write image archive [default: ./{uuid}.tar.gz]
    #[arg(short, long)]
    out: Option<String>,
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    println!("{:?}", cli.command);
    match &cli.command {
        Commands::GetManifest(args) => {
            Client::new(args.host.clone())
                .get_and_save_manifest(uuid::Uuid::parse_str(&args.uuid).unwrap(), args.out.clone())
                .unwrap();
            let manifest = Client::new(args.host.clone())
                .get_manifest(uuid::Uuid::parse_str(&args.uuid).unwrap())
                .unwrap();
            info!("Manifest: {:?}", manifest);
        }
        Commands::GetImage(args) => {
            Client::new(args.host.clone())
                .get_and_save_image(uuid::Uuid::parse_str(&args.uuid).unwrap(), args.out.clone())
                .unwrap();
            let image_bytes = Client::new(args.host.clone())
                .get_image(uuid::Uuid::parse_str(&args.uuid).unwrap())
                .unwrap();

            info!("Image size {}", image_bytes.len());
        }
    }
}
