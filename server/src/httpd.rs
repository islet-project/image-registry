use crate::{registry::Registry, Config};
use axum::{
    extract, http,
    response::IntoResponse,
    routing, Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;


type SafeReg = Arc<RwLock<Registry>>;

static NOT_FOUND: (http::StatusCode, &'static str) =
    (http::StatusCode::NOT_FOUND, "In the beginning there was darkness");

pub async fn run(reg: Registry)
{
    let reg = Arc::new(RwLock::new(reg));

    let http_json = format!("/{}/*file", Config::readu().http);
    let app = Router::new()
        .route(&http_json, routing::get(http_get))
        .with_state(reg)
        .fallback(fallback)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let address = format!("0.0.0.0:{}", Config::readu().port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn fallback() -> (http::StatusCode, &'static str)
{
    NOT_FOUND
}

async fn http_get(extract::Path(file): extract::Path<String>,
                  extract::State(reg): extract::State<SafeReg>)
                  -> impl IntoResponse
{
    let v: Vec<&str> = file.split('.').collect();
    if v.len() != 2 {
        return NOT_FOUND.into_response();
    }

    let reg = reg.read().await;
    match v[1].to_lowercase().as_str() {
        "json" => {
            let uuid = Uuid::parse_str(&v[0]).unwrap_or(Uuid::default());
            if !reg.contains_key(&uuid) {
                (http::StatusCode::NOT_FOUND, "Manifest not found").into_response()
            } else {
                Json(&reg[&uuid].manifest).into_response()
            }
        },
        "tgz" | "tar.gz" | "tbz" | "tar.bz2" | "zip" => {
            (http::StatusCode::NOT_FOUND, "Image not found").into_response()
        }
        _ => NOT_FOUND.into_response()
    }
}
