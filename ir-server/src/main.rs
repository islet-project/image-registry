use ir_server::*;

use clap::Parser;
use log::{debug, error, info};

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
    tls: ConfigProtocol,

    /// server port
    #[arg(short, long, default_value_t = 1337)]
    port: u16,

    /// RA-TLS: Veraison verification service host
    #[arg(short = 'u', long, default_value = "https://localhost:8080")]
    veraison_url: String,

    /// RA-TLS: Veraisons public key, none to use {crate_root}/ratls/pkey.jwk
    #[arg(short = 'v', long)]
    veraison_pubkey: Option<String>,

    /// RA-TLS: JSON containing reference values, none to use {crate_root}/ratls/example.json
    #[arg(short = 'j', long)]
    reference_json: Option<String>,
}

#[tokio::main]
async fn main() -> ir_server::RegistryResult<()>
{
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    // initialize and setup the config singleton
    {
        let mut config = Config::writeu();

        config.set_server_root(cli.root.as_deref())?;
        config.set_server_cert(cli.cert.as_deref())?;
        config.set_server_key(cli.key.as_deref())?;
        config.set_veraison_key(cli.veraison_pubkey.as_deref())?;
        config.set_reference_json(cli.reference_json.as_deref())?;

        config.veraison_url = cli.veraison_url;
        config.port = cli.port;
        config.tls = cli.tls;
    }

    debug!("{:#?}", ir_server::Config::readu());

    let reg = OciRegistry::import(&Config::readu().root)?;
    debug!("{:#?}", reg);
    reg.log_summary();

    info!("Launching the HTTP(S) server");
    if let Result::Err(e) = httpd_run(reg).await {
        error!("{}", e);
    }

    Ok(())
}
