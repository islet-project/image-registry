use axum::{body, extract, http, response::IntoResponse, routing, Json, Router};
use log::{debug, info};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::config::{Config, Protocol};
use crate::registry::ImageRegistry;
use crate::tls;
use crate::RegistryResult;

static NOT_FOUND: (http::StatusCode, &str) = (
    http::StatusCode::NOT_FOUND,
    "In the beginning there was darkness... Or was it 404? I can't remember.",
);

type SafeReg = Arc<RwLock<dyn ImageRegistry>>;

pub async fn run<T: ImageRegistry + 'static>(reg: T) -> RegistryResult<()>
{
    let reg = Arc::new(RwLock::new(reg));

    let app = Router::new()
        .route("/v2/", routing::get(get_support))
        .route("/v2/:name/tags/list", routing::get(get_tags))
        .route(
            "/v2/:name/manifests/:reference",
            routing::get(get_manifest).head(head_manifest),
        )
        .route(
            "/v2/:name/blobs/:digest",
            routing::get(get_blob).head(head_blob),
        )
        .with_state(reg)
        .fallback(fallback)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let address = format!("0.0.0.0:{}", Config::readu().port);
    debug!("Binding address: {}", address);
    let listener = tokio::net::TcpListener::bind(address).await?;

    let tls = { Config::readu().tls.clone() };
    match tls {
        Protocol::NoTls => axum::serve(listener, app).await?,
        Protocol::Tls => tls::serve_tls(listener, app).await?,
        Protocol::RaTls => tls::serve_ratls(listener, app).await?,
    }

    Ok(())
}

async fn fallback() -> (http::StatusCode, &'static str)
{
    NOT_FOUND
}

#[allow(dead_code)]
async fn get_support(extract::State(_reg): extract::State<SafeReg>) -> impl IntoResponse
{
    (http::StatusCode::NOT_IMPLEMENTED, "NI: GET support").into_response()
}

#[allow(dead_code)]
async fn get_tags(extract::State(_reg): extract::State<SafeReg>) -> impl IntoResponse
{
    (http::StatusCode::NOT_IMPLEMENTED, "NI: GET tags").into_response()
}

#[allow(dead_code)]
async fn get_manifest(
    extract::State(_reg): extract::State<SafeReg>,
    extract::Path((name, reference)): extract::Path<(String, String)>,
) -> impl IntoResponse
{
    let msg = format!("NI: GET manifest; name={}, reference={}", name, reference);
    (http::StatusCode::NOT_IMPLEMENTED, msg).into_response()
}

#[allow(dead_code)]
async fn head_manifest(
    extract::State(_reg): extract::State<SafeReg>,
    extract::Path((name, reference)): extract::Path<(String, String)>,
) -> impl IntoResponse
{
    let msg = format!("NI: HEAD manifest; name={}, reference={}", name, reference);
    (http::StatusCode::NOT_IMPLEMENTED, msg).into_response()
}

#[allow(dead_code)]
async fn get_blob(
    extract::State(_reg): extract::State<SafeReg>,
    extract::Path((name, digest)): extract::Path<(String, String)>,
) -> impl IntoResponse
{
    let msg = format!("NI: GET blob; name={}, digest={}", name, digest);
    (http::StatusCode::NOT_IMPLEMENTED, msg).into_response()
}

#[allow(dead_code)]
async fn head_blob(
    extract::State(_reg): extract::State<SafeReg>,
    extract::Path((name, digest)): extract::Path<(String, String)>,
) -> impl IntoResponse
{
    let msg = format!("NI: HEAD blob; name={}, digest={}", name, digest);
    (http::StatusCode::NOT_IMPLEMENTED, msg).into_response()
}

#[allow(dead_code)]
async fn http_get(
    extract::Path(file): extract::Path<String>,
    extract::State(reg): extract::State<SafeReg>,
) -> impl IntoResponse
{
    let v: Vec<&str> = file.split('.').collect();
    if v.len() != 2 {
        return NOT_FOUND.into_response();
    }

    let uuid = Uuid::parse_str(v[0]).unwrap_or_default();
    let registry = reg.read().await;
    match v[1].to_lowercase().as_str() {
        "json" => {
            if let Some(manifest) = registry.get_manifest(&uuid) {
                info!("Manifest for {} found and served", uuid);
                Json(manifest).into_response()
            } else {
                info!("Manifest for {} not found", uuid);
                (http::StatusCode::NOT_FOUND, "Manifest not found").into_response()
            }
        }
        ext @ "tgz" => {
            if let Some(stream) = registry.get_image(&uuid).await {
                let body = body::Body::from_stream(stream);

                let headers = [
                    (http::header::CONTENT_TYPE, "application/octet-stream"),
                    (
                        http::header::CONTENT_DISPOSITION,
                        &format!("attachment; filename=\"{}.{}\"", uuid, ext),
                    ),
                ];

                info!("Image for {} found and served", uuid);
                (headers, body).into_response()
            } else {
                info!("Image for {} not found", uuid);
                (http::StatusCode::NOT_FOUND, "Image not found").into_response()
            }
        }
        _ => NOT_FOUND.into_response(),
    }
}
