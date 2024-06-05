#![allow(dead_code)]

use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub const DEFAULT_SERVER: &str = "registry";
pub const DEFAULT_DATABASE: &str = "database.yaml";

pub struct Config
{
    pub root: String,
    pub server: String,
    pub database: String,
    pub port: u16,

    #[allow(dead_code)]
    block: (),
}

impl Config
{
    fn new() -> Self
    {
        Config {
            root: String::new(),
            server: String::new(),
            database: String::new(),
            port: 0,
            block: (),
        }
    }

    pub fn set_server_root(&mut self, root: &str) -> std::io::Result<()>
    {
        let root = std::fs::canonicalize(root)?;
        self.root = root.to_string_lossy().to_string();
        let server = root.join(DEFAULT_SERVER);
        self.server = server.to_string_lossy().to_string();
        self.database = server.join(DEFAULT_DATABASE).to_string_lossy().to_string();

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
