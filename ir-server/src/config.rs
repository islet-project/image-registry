use std::fs;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::error::RegistryError;
use crate::{utils, RegistryResult};

pub const DEFAULT_SERVER: &str = "../registry";
pub const DEFAULT_CERT: &str = "certs/server.crt";
pub const DEFAULT_KEY: &str = "certs/server.key";
pub const DEFAULT_RATLS_VERAISON_KEY: &str = "ratls/pkey.jwk";
pub const DEFAULT_RATLS_REFERENCE_JSON: &str = "ratls/example.json";

macro_rules! err {
    ($($arg:tt)+) => (Err(RegistryError::Config(format!($($arg)+))))
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum Protocol
{
    #[default]
    NoTls,
    Tls,
    RaTls,
}

#[derive(Debug)]
#[non_exhaustive]
pub struct Config
{
    pub root: String,
    pub cert: String,
    pub key: String,
    pub tls: Protocol,
    pub port: u16,
    pub veraison_url: String,
    pub veraison_pubkey: String,
    pub reference_json: String,
}

impl Config
{
    fn new() -> Self
    {
        Config {
            root: String::new(),
            cert: String::new(),
            key: String::new(),
            tls: Protocol::default(),
            port: 0,
            veraison_url: String::new(),
            veraison_pubkey: String::new(),
            reference_json: String::new(),
        }
    }

    pub fn set_server_root(&mut self, root: Option<&str>) -> RegistryResult<()>
    {
        let base = match root {
            Some(r) => fs::canonicalize(r).or(err!("Server root path \"{}\" doesn't exist", r))?,
            None => fs::canonicalize(utils::get_crate_root())?.join(DEFAULT_SERVER),
        };
        self.root = base.to_string_lossy().to_string();

        Ok(())
    }

    pub fn set_server_cert(&mut self, cert: Option<&str>) -> RegistryResult<()>
    {
        let base = match cert {
            Some(c) => fs::canonicalize(c).or(err!("Server cert path \"{}\" doesn't exist", c))?,
            None => fs::canonicalize(utils::get_crate_root())?.join(DEFAULT_CERT),
        };
        self.cert = base.to_string_lossy().to_string();

        Ok(())
    }

    pub fn set_server_key(&mut self, key: Option<&str>) -> RegistryResult<()>
    {
        let base = match key {
            Some(k) => fs::canonicalize(k).or(err!("Server key path \"{}\" doesn't exist", k))?,
            None => fs::canonicalize(utils::get_crate_root())?.join(DEFAULT_KEY),
        };
        self.key = base.to_string_lossy().to_string();

        Ok(())
    }

    pub fn set_veraison_key(&mut self, key: Option<&str>) -> RegistryResult<()>
    {
        let base = match key {
            Some(k) => fs::canonicalize(k).or(err!("Veraizon key path \"{}\" doesn't exist", k))?,
            None => fs::canonicalize(utils::get_crate_root())?.join(DEFAULT_RATLS_VERAISON_KEY),
        };
        self.veraison_pubkey = base.to_string_lossy().to_string();

        Ok(())
    }

    pub fn set_reference_json(&mut self, key: Option<&str>) -> RegistryResult<()>
    {
        let base = match key {
            Some(j) => fs::canonicalize(j)
                .or(err!("Veraizon reference JSON path \"{}\" doesn't exist", j))?,
            None => fs::canonicalize(utils::get_crate_root())?.join(DEFAULT_RATLS_REFERENCE_JSON),
        };
        self.reference_json = base.to_string_lossy().to_string();

        Ok(())
    }
}

///////////////////////////
//    SINGLETON BELOW    //
//    HERE BE DRAGONS    //
///////////////////////////

// First usage of singleton must happen before any threads are created

type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;

impl Config
{
    pub fn get() -> &'static RwLock<Config>
    {
        static mut CONFIG: Option<RwLock<Config>> = None;

        unsafe {
            if (*std::ptr::addr_of!(CONFIG)).is_none() {
                CONFIG.replace(RwLock::new(Config::new()));
            }

            CONFIG.as_ref().unwrap()
        }
    }

    pub fn read() -> LockResult<RwLockReadGuard<'static, Config>>
    {
        Config::get().read()
    }

    pub fn write() -> LockResult<RwLockWriteGuard<'static, Config>>
    {
        Config::get().write()
    }

    pub fn readu() -> RwLockReadGuard<'static, Config>
    {
        Config::read().unwrap()
    }

    pub fn writeu() -> RwLockWriteGuard<'static, Config>
    {
        Config::write().unwrap()
    }
}
