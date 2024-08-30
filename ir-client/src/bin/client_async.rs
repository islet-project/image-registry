use std::sync::Arc;

use ir_client::oci::{client::Client, reference::{Digest, Reference, Tag}};
use ir_client::{config::Config, verify_digest};

use clap::{Args, Parser, Subcommand, ValueEnum};
use log::info;
use ratls::{load_root_cert_store, RaTlsCertResolver, TokenFromFile};
use tokio::io::AsyncReadExt;

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
    GetBlob(GetBlobArgs),
    ListTags(ListTagsArgs),
}

#[derive(Args, Debug)]
struct GetManifestArgs {
    /// Repository namespace (application name)
    #[arg(short, long)]
    app_name: String,

    /// Reference of manifest [digest or tag]
    #[arg(short, long)]
    reference: String,

    /// filename to write image JSON [default: ./{uuid}.json]
    #[arg(short, long)]
    out: Option<String>,
}

#[derive(Args, Debug)]
struct GetBlobArgs {
    /// Repository namespace (application name)
    #[arg(short, long)]
    app_name: String,

    // Digest of blob
    #[arg(short, long)]
    digest: String,

    /// filename to write image archive [default: ./{uuid}.tar.gz]
    #[arg(short, long)]
    out: Option<String>,
}

#[derive(Args, Debug)]
struct ListTagsArgs {
    /// Repository namespace (application name)
    #[arg(short, long)]
    app_name: String,

    /// List only N tags
    #[arg(short, long)]
    n: Option<usize>,

    /// Start listing tags after LAST
    #[arg(short, long)]
    last: Option<String>,
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

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    let config = build_config(cli.conn);
    let client = Client::from_config(config).unwrap();

    println!("{:?}", cli.command);
    match &cli.command {
        Commands::GetManifest(args) => {
            let reference = Reference::try_from(args.reference.as_str()).unwrap();
            let manifest = client
                .get_manifest(&args.app_name, reference)
                .await
                .unwrap();
            info!("{}", manifest);
        }
        Commands::GetBlob(args) => {
            let digest = Digest::try_from(args.digest.as_str()).unwrap();
            let mut blob_reader = client
                .get_blob_reader(&args.app_name, digest)
                .await
                .unwrap();

            let mut buf = Vec::new();
            blob_reader.read_to_end(&mut buf).await.unwrap();
            info!("Image size = {}, content-length: {:?}", buf.len(), blob_reader.len());
            let digest = blob_reader.digest().as_ref().unwrap();
            info!("Blob digest: {}", digest.to_string());
            verify_digest(digest, &buf);
        },
        Commands::ListTags(args) => {
            let last = args.last.clone().map(|user_tag| Tag::try_from(user_tag.as_str()).unwrap());
            let tag_list = client.list_tags_with_options(&args.app_name, args.n, last).await.unwrap();

            info!("{}", serde_json::to_string_pretty(&tag_list).unwrap());
        }
    }
}
