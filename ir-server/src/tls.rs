use axum::{extract::Request, Router};
use futures_util::pin_mut;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::TlsAcceptor;
use tower_service::Service;

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

// For details on the code see here:
// https://github.com/tokio-rs/axum/blob/main/examples/low-level-rustls/src/main.rs
pub async fn serve(listener: TcpListener, app: Router) -> RegistryResult<()>
{
    let rustls_config = tls_server_config()?;
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
