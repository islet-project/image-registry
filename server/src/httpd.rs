use crate::{registry::Registry, Config};
use axum::{
    extract, http,
    response::{self, IntoResponse},
    routing, Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use uuid::Uuid;


type SafeReg = Arc<RwLock<Registry>>;

pub async fn run(reg: Registry)
{
    let reg = Arc::new(RwLock::new(reg));

    let http_json = format!("/{}/:id", Config::readu().http);
    let app = Router::new()
        .route(&http_json, routing::get(http_get))
        .with_state(reg)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let address = format!("0.0.0.0:{}", Config::readu().port);
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn http_get(extract::Path(id): extract::Path<String>,
                  extract::State(reg): extract::State<SafeReg>)
                  -> impl response::IntoResponse
{
    let reg = reg.read().await;
    let uuid = Uuid::parse_str(&id).unwrap_or(Uuid::default());
    if !reg.contains_key(&uuid) {
        (http::StatusCode::NOT_FOUND, "Requested UUID not found").into_response()
    } else {
        Json(&reg[&uuid].manifest).into_response()
    }
}
