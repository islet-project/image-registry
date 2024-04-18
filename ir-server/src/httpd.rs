use crate::{registry::ImageRegistry, Config, GenericResult};
use axum::{body, extract, http, response::IntoResponse, routing, Json, Router};
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

static NOT_FOUND: (http::StatusCode, &'static str) = (
    http::StatusCode::NOT_FOUND,
    "In the beginning there was darkness",
);

type SafeReg = Arc<RwLock<dyn ImageRegistry>>;

pub async fn run<T: ImageRegistry + 'static>(reg: T) -> GenericResult<()>
{
    let reg = Arc::new(RwLock::new(reg));

    let http_json = format!("/{}/*file", Config::readu().http);
    let app = Router::new()
        .route(&http_json, routing::get(http_get))
        .with_state(reg)
        .fallback(fallback)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let address = format!("0.0.0.0:{}", Config::readu().port);
    let listener = tokio::net::TcpListener::bind(address).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn fallback() -> (http::StatusCode, &'static str)
{
    NOT_FOUND
}

async fn http_get(
    extract::Path(file): extract::Path<String>,
    extract::State(reg): extract::State<SafeReg>,
) -> impl IntoResponse
{
    let v: Vec<&str> = file.split('.').collect();
    if v.len() != 2 {
        return NOT_FOUND.into_response();
    }

    let uuid = Uuid::parse_str(&v[0]).unwrap_or(Uuid::default());
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