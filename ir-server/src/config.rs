#![allow(dead_code)]

use std::fs;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::utils;

pub const DEFAULT_SERVER: &str = "registry";
pub const DEFAULT_CERT: &str = "certs/server.crt";
pub const DEFAULT_KEY: &str = "certs/server.key";
pub const DEFAULT_DATABASE: &str = "database.yaml";

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum Protocol
{
    #[default]
    NoTls,
    Tls,
    RaTls,
}

pub struct Config
{
    pub root: String,
    pub cert: String,
    pub key: String,
    pub database: String,
    pub port: u16,
    pub proto: Protocol,

    #[allow(dead_code)]
    block: (),
}

impl Config
{
    fn new() -> Self
    {
        Config {
            root: String::new(),
            cert: String::new(),
            key: String::new(),
            database: String::new(),
            port: 0,
            proto: Protocol::default(),
            block: (),
        }
    }

    pub fn set_server_root(&mut self, root: Option<&str>) -> std::io::Result<()>
    {
        let base = match root {
            Some(r) => fs::canonicalize(r)?,
            None => fs::canonicalize(&utils::get_crate_root())?.join(DEFAULT_SERVER),
        };
        self.root = base.to_string_lossy().to_string();
        self.database = base.join(DEFAULT_DATABASE).to_string_lossy().to_string();

        Ok(())
    }

    pub fn set_server_cert(&mut self, cert: Option<&str>) -> std::io::Result<()>
    {
        let base = match cert {
            Some(c) => fs::canonicalize(c)?,
            None => fs::canonicalize(&utils::get_crate_root())?.join(DEFAULT_CERT),
        };
        self.cert = base.to_string_lossy().to_string();

        Ok(())
    }

    pub fn set_server_key(&mut self, key: Option<&str>) -> std::io::Result<()>
    {
        let base = match key {
            Some(k) => fs::canonicalize(k)?,
            None => fs::canonicalize(&utils::get_crate_root())?.join(DEFAULT_KEY),
        };
        self.key = base.to_string_lossy().to_string();

        Ok(())
    }
}

///////////////////////////
//    SINGLETON BELOW    //
//    HERE BE DRAGONS    //
///////////////////////////

// First usage of singleton must happen before any threads are created

#[allow(dead_code)]
type LockResult<Guard> = Result<Guard, PoisonError<Guard>>;

impl Config
{
    pub fn get() -> &'static RwLock<Config>
    {
        static mut CONFIG: Option<RwLock<Config>> = None;

        unsafe {
            if let None = *std::ptr::addr_of!(CONFIG) {
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
