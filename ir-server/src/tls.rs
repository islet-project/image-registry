use axum::{extract::Request, Router};
use futures_util::pin_mut;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use log::debug;
use ratls::{ChainVerifier, InternalTokenVerifier, RaTlsCertVeryfier};
use realm_verifier::{parser_json::parse_value, RealmVerifier};
use std::{fs::File, io::BufReader, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::rustls::crypto::ring::default_provider;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tower_service::Service;
use veraison_verifier::VeraisonTokenVerifer;

use crate::config::Config;
use crate::utils;
use crate::RegistryResult;

enum TLSConfig<'a>
{
    TLS(Arc<ServerConfig>),
    RaTLS(RaTLS<'a>),
}

struct RaTLS<'a>
{
    pub certs: Vec<CertificateDer<'a>>,
    pub priv_key: PrivateKeyDer<'a>,
    pub client_token_verifier: Arc<dyn InternalTokenVerifier>,
}

impl TLSConfig<'static>
{
    pub fn get_rustls_config(&self) -> RegistryResult<Arc<ServerConfig>>
    {
        match self {
            Self::TLS(config) => Ok(config.clone()),
            Self::RaTLS(ra_tls) => {
                let rustls_config = ServerConfig::builder()
                    .with_client_cert_verifier(Arc::new(RaTlsCertVeryfier::from_token_verifier(
                        ra_tls.client_token_verifier.clone(),
                    )))
                    .with_single_cert(ra_tls.certs.clone(), ra_tls.priv_key.clone_key())?;
                Ok(Arc::new(rustls_config))
            }
        }
    }
}

fn tls_server_config() -> RegistryResult<TLSConfig<'static>>
{
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            utils::load_certificates_from_pem(&Config::readu().cert)?,
            utils::load_private_key_from_file(&Config::readu().key)?,
        )?;

    Ok(TLSConfig::TLS(Arc::new(config)))
}

fn ratls_server_config() -> RegistryResult<TLSConfig<'static>>
{
    let json_reader = BufReader::new(File::open(&Config::readu().reference_json)?);
    let mut reference_json: serde_json::Value = serde_json::from_reader(json_reader)?;
    let reference_measurements = parse_value(reference_json["realm"]["reference-values"].take())?;

    let client_token_verifier = Arc::new(ChainVerifier::new(vec![
        Arc::new(VeraisonTokenVerifer::new(
            &Config::readu().veraison_url,
            std::fs::read_to_string(&Config::readu().veraison_pubkey)?,
            None,
        )?),
        Arc::new(RealmVerifier::init(reference_measurements.clone())),
    ]));
    let certs = utils::load_certificates_from_pem(&Config::readu().cert)?;
    let priv_key = utils::load_private_key_from_file(&Config::readu().key)?;

    Ok(TLSConfig::RaTLS(RaTLS {
        client_token_verifier,
        certs,
        priv_key,
    }))
}

pub async fn serve_tls(listener: TcpListener, app: Router) -> RegistryResult<()>
{
    debug!("Initializing TLS");

    default_provider()
        .install_default()
        .expect("Could not install CryptoProvider");

    let tls_config = tls_server_config()?;
    serve_internal(listener, app, tls_config).await
}

pub async fn serve_ratls(listener: TcpListener, app: Router) -> RegistryResult<()>
{
    debug!("Initializing RA-TLS");

    default_provider()
        .install_default()
        .expect("Could not install CryptoProvider");

    let tls_config = ratls_server_config()?;
    serve_internal(listener, app, tls_config).await
}

// For details on the code see here:
// https://github.com/tokio-rs/axum/blob/main/examples/low-level-rustls/src/main.rs
async fn serve_internal(
    listener: TcpListener,
    app: Router,
    tls_config: TLSConfig<'static>,
) -> RegistryResult<()>
{
    pin_mut!(listener);

    loop {
        let tower_service = app.clone();

        let (cnx, addr) = listener.accept().await?;
        let tls_acceptor = TlsAcceptor::from(tls_config.get_rustls_config()?);

        tokio::spawn(async move {
            let Ok(stream) = tls_acceptor.accept(cnx).await else {
                log::error!("error during tls handshake connection from {}", addr);
                return;
            };

            let stream = TokioIo::new(stream);

            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            let ret = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(stream, hyper_service)
                .await;

            if let Err(err) = ret {
                log::warn!("error serving connection from {}: {}", addr, err);
            }
        });
    }
}
