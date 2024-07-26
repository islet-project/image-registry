use std::sync::Arc;

use rustls::{client::ResolvesClientCert, ClientConfig, RootCertStore};

pub enum ConnectionMode {
    None,
    RusTLS(ClientConfig),
    RaTLS(ClientConfig),
    CustomTLS(ClientConfig),
}

pub struct Config {
    pub(crate) host: String,
    pub(crate) mode: ConnectionMode,
}

impl Config {
    pub fn builder() -> ConfigBuilder<WantsConfig> {
        ConfigBuilder {
            state: WantsConfig {}
        }
    }
}

impl ConnectionMode {
    pub fn is_secure(&self) -> bool {
        if let ConnectionMode::None = self {
            return false;
        }
        return true;
    }

    pub fn into_rustls_config(self) -> Option<ClientConfig> {
        match self {
            ConnectionMode::None => None,
            ConnectionMode::CustomTLS(config) => Some(config),
            ConnectionMode::RaTLS(config) => Some(config),
            ConnectionMode::RusTLS(config) => Some(config),
        }
    }
}

fn scheme_from_mode(mode: &ConnectionMode) -> &'static str {
    match mode {
        ConnectionMode::None => HOST_PROTOCOL_NONSECURE_SCHEME,
        ConnectionMode::CustomTLS(_) => HOST_PROTOCOL_SECURE_SCHEME,
        ConnectionMode::RaTLS(_) => HOST_PROTOCOL_SECURE_SCHEME,
        ConnectionMode::RusTLS(_) => HOST_PROTOCOL_SECURE_SCHEME,
    }
}

impl Config {
    pub fn scheme(&self) -> &'static str {
        scheme_from_mode(&self.mode)
    }
}

// impl Config {
//     pub fn no_tls() -> Self {
//         Self { mode: TLSMode::None }
//     }

//     pub fn rustls_no_auth(root_cert_store: RootCertStore) -> Self {
//         let rustls_config = ClientConfig::builder()
//             .with_root_certificates(root_cert_store)
//             .with_no_client_auth();
//         Self { mode: TLSMode::RusTLS(rustls_config) }
//     }

//     pub fn ratls(root_cert_store: RootCertStore, resolver: Arc<dyn ResolvesClientCert>) -> Self {
//         let rustls_config = ClientConfig::builder()
//             .with_root_certificates(root_cert_store)
//             .with_client_cert_resolver(resolver);
//         Self { mode: TLSMode::RaTLS(rustls_config) }
//     }

//     pub fn rustls_preconfig(rustls_config: ClientConfig) -> Self {
//         Self { mode: TLSMode::Custom(rustls_config) }
//     }

//     pub fn into_rustls_config(self) -> Option<ClientConfig> {
//         match self.mode {
//             TLSMode::None => None,
//             TLSMode::Custom(config) => Some(config),
//             TLSMode::RaTLS(config) => Some(config),
//             TLSMode::RusTLS(config) => Some(config),
//         }
//     }
// }

pub struct ConfigBuilder<S> {
    state: S,
}

pub struct WantsConfig {}

pub struct WantsHost {
    pub(crate) host: String,
}

impl ConfigBuilder<WantsConfig> {
    pub fn host(self, host: String) -> ConfigBuilder<WantsHost> {
        ConfigBuilder {
            state: WantsHost {
                host
            }
        }
    }
}

impl ConfigBuilder<WantsHost> {
    pub fn no_tls(self) -> Config {
        Config {
            host: self.state.host,
            mode: ConnectionMode::None,
        }
    }

    pub fn rustls_no_auth(self, root_cert_store: RootCertStore) -> Config {
        let rustls_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        Config {
            mode: ConnectionMode::RusTLS(rustls_config),
            host: self.state.host,

        }
    }

    pub fn ratls(self, root_cert_store: RootCertStore, resolver: Arc<dyn ResolvesClientCert>) -> Config {
        let rustls_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_client_cert_resolver(resolver);
        Config {
            mode: ConnectionMode::RaTLS(rustls_config),
            host: self.state.host,

        }
    }

    pub fn rustls_preconfig(self, rustls_config: ClientConfig) -> Config {
        Config {
            mode: ConnectionMode::CustomTLS(rustls_config),
            host: self.state.host,
        }
    }
}
