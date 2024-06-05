mod config;
mod error;
mod httpd;
mod registry;
mod tls;
mod utils;

use clap::Parser;
use config::Config;
use log::info;
use registry::Registry;

type RegistryResult<T> = Result<T, error::RegistryError>;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli
{
    /// runtime server root directory, none to use {crate_root}/registry
    #[arg(short, long)]
    root: Option<String>,

    /// path to server certificate, none to use {crate_root}/certs/server.crt
    #[arg(short, long)]
    cert: Option<String>,

    /// path to server private key, none to use {crate_root}/certs/server.key
    #[arg(short, long)]
    key: Option<String>,

    /// TLS variant to use
    #[arg(short, long, default_value_t, value_enum)]
    tls: config::Protocol,

    /// server port
    #[arg(short, long, default_value_t = 8888)]
    port: u16,

    /// generate new example registry removing previous one if exists
    #[arg(short, long)]
    gen: bool,
}

#[tokio::main]
async fn main() -> RegistryResult<()>
{
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    // initialize and setup the config singleton
    {
        let mut config = Config::writeu();

        config.set_server_root(cli.root.as_deref())?;
        config.set_server_cert(cli.cert.as_deref())?;
        config.set_server_key(cli.key.as_deref())?;

        config.port = cli.port;
        config.proto = cli.tls;
    }

    if cli.gen {
        info!("Generating an example image registry");
        Registry::generate_example()?;
        return Ok(());
    }

    info!("Server root: {}", Config::readu().root);
    info!("Server certificates: {}", Config::readu().cert);
    info!("Server private key: {}", Config::readu().key);

    let reg = Registry::import()?;
    info!("{:?}", reg);

    info!("Launching the HTTP(S) server");
    httpd::run(reg).await?;

    Ok(())
}
