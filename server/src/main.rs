mod config;
mod utils;

use clap::Parser;
use config::Config;

type GenericResult = Result<(), Box<dyn std::error::Error>>;


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
}

fn main() -> GenericResult {
    let cli = Cli::parse();

    // initialize and setup the config singleton
    {
        if let Some(root) = cli.root {
            Config::writeu().set_server_root(&root)?;
        } else {
            Config::writeu().set_server_root(&utils::get_crate_root())?;
        }

        Config::writeu().port = cli.port;
    }

    println!("Server root: {}", Config::readu().root);

    Ok(())
}
