use std::path::Path;
use std::sync::Arc;

use ir_client::oci::reference::Reference;
use ir_client::{client::Client, config::Config};

use clap::{Args, Parser, ValueEnum};
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

    /// Repository namespace (application name)
    #[arg(short = 'n', long)]
    app_name: String,

    /// Reference of image manifest [digest or tag]
    #[arg(short, long)]
    reference: String,

    /// directory to unpack the image
    #[arg(short, long)]
    dest: String,
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

    let reference = Reference::try_from(cli.reference.as_str()).unwrap();

    let image_info = client.get_image_info(&cli.app_name, reference).await.unwrap();

    let tmp = Path::new(cli.dest.as_str()).parent().unwrap().to_path_buf();
    client.unpack_image(&image_info, &cli.dest, tmp).await.unwrap();
}
