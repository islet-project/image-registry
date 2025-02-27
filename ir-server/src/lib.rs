mod config;
mod error;
mod httpd;
mod oci;
mod registry;
mod tls;
mod utils;

pub type RegistryResult<T> = Result<T, error::RegistryError>;

pub use httpd::run as httpd_run;
pub use config::Config;
pub use config::Protocol as ConfigProtocol;
pub use oci::Registry as OciRegistry;
