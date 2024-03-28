mod config;
mod registry;
mod utils;

use clap::Parser;
use config::Config;
use log::info;

type GenericResult<T> = Result<T, Box<dyn std::error::Error>>;


#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli
{
    /// runtime server root directory, none to use {crate_root}/server
    #[arg(short, long)]
    root: Option<String>,

    /// server port
    #[arg(short, long, default_value_t = 8888)]
    port: u16,

    /// generate new example registry removing previous one if exists
    #[arg(short, long)]
    gen: bool,
}

fn main() -> GenericResult<()>
{
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let cli = Cli::parse();

    // handle cmd line option, initialize and setup the config singleton
    {
        if let Some(root) = cli.root {
            Config::writeu().set_server_root(&root)?;
        } else {
            Config::writeu().set_server_root(&utils::get_crate_root())?;
        }

        Config::writeu().port = cli.port;

        if cli.gen {
            registry::generate_registry()?;
        }
    }

    info!("Server root: {}", Config::readu().root);

    let reg = registry::parse_to_hashmap(registry::deserialize()?)?;
    info!("{:#?}", reg);

    Ok(())
}
