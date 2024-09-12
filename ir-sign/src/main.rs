mod crypto;
mod digest;
mod error;
mod oci;
mod subcmds;
mod utils;

use clap::{Parser, Subcommand};

type SignerResult<T> = Result<T, error::SignerError>;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli
{
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands
{
    /// Generate ECDSA P384 private/public key
    GenKey
    {
        /// Path to the output file
        #[arg(short, long)]
        output: String,
    },

    /// Extract public key from a private one
    ExtractPublic
    {
        /// Path to the private key
        #[arg(short, long)]
        input: String,

        /// Path to the public key
        #[arg(short, long)]
        output: String,
    },

    /// Sign a file
    Sign
    {
        /// Path to the private key
        #[arg(short, long)]
        key: String,

        /// Path to the file to sign
        #[arg(short, long)]
        file: String,

        /// Path to the signature file
        #[arg(short, long)]
        signature: String,
    },

    /// Verify a signature
    Verify
    {
        /// Path to the public key
        #[arg(short, long)]
        key: String,

        /// Path to the signed file
        #[arg(short, long)]
        file: String,

        /// Path to the signature file
        #[arg(short, long)]
        signature: String,
    },

    /// Sign a config for a specific manifest
    SignConfig
    {
        /// Path to the registry
        #[arg(short, long, default_value = "../registry")]
        registry: String,

        /// Name of the application in registry
        #[arg(short, long, default_value = "com.samsung.example.app")]
        app: String,

        /// Digest of the manifest to sign
        #[arg(short, long)]
        digest: String,

        /// Path to the vendor private key
        #[arg(short, long)]
        vendor_prv: String,

        /// Path to the signature of vendor public key signed by root-ca
        #[arg(short = 's', long)]
        vendor_pub_signature: String,

        /// Path to the root-ca public-key
        #[arg(short, long)]
        ca_pub: String,
    },

    /// Rehash a file and rename it
    RehashFile
    {
        /// Path to the registry
        #[arg(short, long, default_value = "../registry")]
        registry: String,

        /// Name of the application in registry
        #[arg(short, long, default_value = "com.samsung.example.app")]
        app: String,

        /// Digest of the file to rehash
        #[arg(short, long)]
        digest: String,
    },

    /// Sign the image/config from a specific manifest and fix the application
    /// You need to pass one of the two:
    ///   1. VENDOR_PUB_SIGNATURE and CA_PUB
    ///   2. CA_PRV
    #[command(verbatim_doc_comment)]
    SignImage
    {
        /// Path to the registry
        #[arg(short, long, default_value = "../registry")]
        registry: String,

        /// Name of the application in registry
        #[arg(short, long, default_value = "com.samsung.example.app")]
        app: String,

        /// Digest of the manifest to sign
        #[arg(short, long)]
        digest: String,

        /// Path to the vendor private key
        #[arg(short, long)]
        vendor_prv: String,

        /// Path to the signature of vendor public key signed by root-ca
        #[arg(short = 's', long)]
        vendor_pub_signature: Option<String>,

        /// Path to the root-ca public-key
        #[arg(short, long)]
        ca_pub: Option<String>,

        /// Path to the root-ca private key
        #[arg(short = 'x', long)]
        ca_prv: Option<String>,
    },
}

fn main() -> SignerResult<()>
{
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("debug"));

    let cli = Cli::parse();

    let Some(command) = cli.command else {
        return Err("You need to pass command".into());
    };

    match command {
        Commands::GenKey { output } => subcmds::cmd_generate_key(&output)?,
        Commands::ExtractPublic { input, output } => subcmds::cmd_extract_public(&input, &output)?,
        Commands::Sign {
            key,
            file,
            signature,
        } => subcmds::cmd_sign_buf(&key, &file, &signature)?,
        Commands::Verify {
            key,
            signature,
            file,
        } => subcmds::cmd_verify_buf(&key, &file, &signature)?,
        Commands::SignConfig {
            registry,
            app,
            digest,
            vendor_prv,
            vendor_pub_signature,
            ca_pub,
        } => subcmds::cmd_sign_config(
            &registry,
            &app,
            &digest,
            &vendor_prv,
            &vendor_pub_signature,
            &ca_pub,
        )?,
        Commands::RehashFile {
            registry,
            app,
            digest,
        } => subcmds::cmd_rehash_file(&registry, &app, &digest)?,
        Commands::SignImage {
            registry,
            app,
            digest,
            vendor_prv,
            vendor_pub_signature,
            ca_pub,
            ca_prv,
        } => subcmds::cmd_sign_image(
            &registry,
            &app,
            &digest,
            &vendor_prv,
            vendor_pub_signature.as_deref(),
            ca_pub.as_deref(),
            ca_prv.as_deref(),
        )?,
    }

    Ok(())
}
