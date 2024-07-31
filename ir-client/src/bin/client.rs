use std::sync::Arc;

use ir_client::{config::Config, Client};

use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
use ratls::{load_root_cert_store, RaTlsCertResolver, TokenFromFile};

#[derive(ValueEnum, Default, Debug, Clone)]
pub enum ConnectionType {
    #[default]
    NoTLS,
    TLS,
    RaTLS,
}

#[derive(Args, Debug)]
#[command(author, version, about)]
struct ConnectionArgs {
    /// Host url address
    #[arg(short = 'a', long, default_value = "localhost:1337")]
    host: String,

    /// Connection type
    #[arg(short, long, default_value_t, value_enum, id = "mode")]
    tls: ConnectionType,

    /// CCA token file in binary CBOR format (used with ra-tls)
    #[arg(long, id = "token.bin", default_value = "./res/token.bin")]
    token: String,

    /// Root certificate file in PEM format (used with tls and ra-tls)
    #[arg(long, id = "root-ca.crt", default_value = "./res/root-ca.crt")]
    root_ca: String,
}

#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(flatten)]
    conn: ConnectionArgs,

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
    /// Uuid of image
    #[arg(short, long)]
    uuid: String,

    /// filename to write image JSON [default: ./{uuid}.json]
    #[arg(short, long)]
    out: Option<String>,
}

#[derive(Args, Debug)]
struct GetImageArgs {
    // Uuid of image
    #[arg(short, long)]
    uuid: String,

    /// filename to write image archive [default: ./{uuid}.tar.gz]
    #[arg(short, long)]
    out: Option<String>,
}

fn build_config(conn: ConnectionArgs) -> Config {
    match conn.tls {
        ConnectionType::NoTLS => {
            Config::builder().host(conn.host).no_tls()
        },
        ConnectionType::TLS => {
            Config::builder()
                .host(conn.host)
                .rustls_no_auth(load_root_cert_store(conn.root_ca).unwrap())
        }
        ConnectionType::RaTLS => {
            Config::builder()
                .host(conn.host)
                .ratls(
                    load_root_cert_store(conn.root_ca).unwrap(),
                    Arc::new(
                        RaTlsCertResolver::from_token_resolver(
                            Arc::new(
                                TokenFromFile::from_path(conn.token).unwrap()
                            )
                        ).unwrap()
                    )
                )
        },
    }
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();
    println!("{:?}", cli);

    let config = build_config(cli.conn);
    let client = Client::from_config(config).unwrap();

    match &cli.command {
        Commands::GetManifest(args) => {
            client.get_and_save_manifest(
                uuid::Uuid::parse_str(&args.uuid).unwrap(),
                args.out.clone()
            ).unwrap();
            let manifest = client
                .get_manifest(uuid::Uuid::parse_str(&args.uuid).unwrap())
                .unwrap();
            info!("Manifest: {:?}", manifest);
        }
        Commands::GetImage(args) => {
            client.get_and_save_image(
                uuid::Uuid::parse_str(&args.uuid).unwrap(),
                args.out.clone()
            ).unwrap();
            let image_bytes = client
                .get_image(uuid::Uuid::parse_str(&args.uuid).unwrap())
                .unwrap();

            info!("Image size {}", image_bytes.len());
        }
    }
}
