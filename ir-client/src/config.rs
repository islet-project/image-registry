use std::sync::Arc;

use rustls::{client::ResolvesClientCert, ClientConfig, RootCertStore};

use crate::service_url::{Scheme, HTTPS_SCHEME, HTTP_SCHEME};

pub(crate) enum ConnectionMode {
    None,
    RusTLS(ClientConfig),
    RaTLS(ClientConfig),
    CustomTLS(ClientConfig),
}

impl ConnectionMode {
    pub fn scheme(&self) -> &'static Scheme {
        match self {
            Self::None => &HTTP_SCHEME,
            Self::CustomTLS(_) => &HTTPS_SCHEME,
            Self::RaTLS(_) => &HTTPS_SCHEME,
            Self::RusTLS(_) => &HTTPS_SCHEME,
        }
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
