use axum::{extract::Request, Router};
use futures_util::pin_mut;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use ratls::{ChainVerifier, InternalTokenVerifier, RaTlsCertVeryfier};
use realm_verifier::{parser_json::parse_value, RealmVerifier};
use std::{fs::File, io::BufReader, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::rustls::crypto::ring::default_provider;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tower_service::Service;
// use veraison_verifier::VeraisonTokenVerifer;

use crate::config::Config;
use crate::utils;
use crate::RegistryResult;

fn tls_server_config() -> RegistryResult<Arc<ServerConfig>>
{
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            utils::load_certificates_from_pem(&Config::readu().cert)?,
            utils::load_private_key_from_file(&Config::readu().key)?,
        )?;

    Ok(Arc::new(config))
}

fn ratls_server_config(
    client_token_verifier: Arc<dyn InternalTokenVerifier>,
) -> RegistryResult<Arc<ServerConfig>>
{
    let config = ServerConfig::builder()
        .with_client_cert_verifier(Arc::new(RaTlsCertVeryfier::from_token_verifier(
            client_token_verifier,
        )))
        .with_single_cert(
            utils::load_certificates_from_pem(&Config::readu().cert)?,
            utils::load_private_key_from_file(&Config::readu().key)?,
        )?;

    Ok(Arc::new(config))
}

pub async fn serve_tls(listener: TcpListener, app: Router) -> RegistryResult<()>
{
    default_provider()
        .install_default()
        .expect("Could not install CryptoProvider");

    let rustls_config = tls_server_config()?;
    serve_internal(listener, app, rustls_config).await
}

pub async fn serve_ratls(listener: TcpListener, app: Router) -> RegistryResult<()>
{
    default_provider()
        .install_default()
        .expect("Could not install CryptoProvider");

    let json_reader = BufReader::new(File::open(&Config::readu().reference_json)?);
    let mut reference_json: serde_json::Value = serde_json::from_reader(json_reader)?;
    let reference_measurements = parse_value(reference_json["realm"]["reference-values"].take())?;

    let client_token_verifier = Arc::new(ChainVerifier::new(vec![
        // Arc::new(VeraisonTokenVerifer::new(
        //     &Config::readu().veraison_url,
        //     &Config::readu().veraison_pubkey,
        // )),
        Arc::new(RealmVerifier::init(reference_measurements)),
    ]));

    let rustls_config = ratls_server_config(client_token_verifier)?;
    serve_internal(listener, app, rustls_config).await
}

// For details on the code see here:
// https://github.com/tokio-rs/axum/blob/main/examples/low-level-rustls/src/main.rs
async fn serve_internal(
    listener: TcpListener,
    app: Router,
    rustls_config: Arc<ServerConfig>,
) -> RegistryResult<()>
{
    let tls_acceptor = TlsAcceptor::from(rustls_config);

    pin_mut!(listener);

    loop {
        let tower_service = app.clone();
        let tls_acceptor = tls_acceptor.clone();

        let (cnx, addr) = listener.accept().await?;

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
