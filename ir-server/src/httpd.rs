use axum::http::HeaderName;
use axum::{body, extract, http, response::IntoResponse, routing, Json, Router};
use axum_extra::headers::Range;
use axum_extra::TypedHeader;
use hyper::Response;
use log::{debug, info};
use serde::Deserialize;
use serde_json::json;
use std::ops::Bound;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::config::{Config, Protocol};
use crate::registry::{ImageRegistry, Payload};
use crate::stream::StreamBytesExt;
use crate::tls;
use crate::RegistryResult;

static NOT_FOUND: (http::StatusCode, &str) = (
    http::StatusCode::NOT_FOUND,
    "In the beginning there was darkness... Or was it 404? I can't remember.",
);

const HEADER_DIGEST: HeaderName = HeaderName::from_static("docker-content-digest");

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

fn serve_file(payload: Payload, range: Option<TypedHeader<Range>>) -> Response<body::Body>
{
    if let Some(TypedHeader(range)) = range {
        let ranges: Vec<_> = range.satisfiable_ranges(payload.size).collect();
        match (ranges.len(), ranges[0]) {
            // very basic implementation of "range: bytes=SKIP-" client header
            (1, (Bound::Included(skip), Bound::Unbounded)) => {
                let body = body::Body::from_stream(payload.stream.skip_bytes(skip as usize));
                let headers = [
                    (http::header::CONTENT_TYPE, &payload.media_type),
                    (
                        http::header::CONTENT_LENGTH,
                        &format!("{}", payload.size - skip),
                    ),
                    (HEADER_DIGEST, &payload.digest),
                    (
                        http::header::CONTENT_RANGE,
                        &format!("bytes {}-{}/{}", skip, payload.size - 1, payload.size),
                    ),
                ];
                (http::StatusCode::PARTIAL_CONTENT, headers, body).into_response()
            }
            // reject all the other variants of partial content or multi-parts
            _ => {
                let headers = [(
                    http::header::CONTENT_RANGE,
                    &format!("bytes */{}", payload.size),
                )];
                let body = body::Body::empty();
                (http::StatusCode::NOT_ACCEPTABLE, headers, body).into_response()
            }
        }
    } else {
        let headers = [
            (http::header::CONTENT_TYPE, &payload.media_type),
            (http::header::CONTENT_LENGTH, &format!("{}", payload.size)),
            (HEADER_DIGEST, &payload.digest),
        ];
        let body = body::Body::from_stream(payload.stream);
        (headers, body).into_response()
    }
}

async fn get_support() -> impl IntoResponse
{
    (http::StatusCode::OK, "OCI Distribution Spec V2 supported").into_response()
}

#[derive(Debug, Deserialize)]
struct TagListParams
{
    n: Option<usize>,
    last: Option<String>,
}

async fn get_tags(
    extract::State(reg): extract::State<SafeReg>,
    extract::Path(name): extract::Path<String>,
    extract::Query(params): extract::Query<TagListParams>,
) -> impl IntoResponse
{
    let registry = reg.read().await;
    let tags = registry.get_tags(&name);

    let Some(mut tags) = tags else {
        return NOT_FOUND.into_response();
    };

    tags.sort_by(|a, b| {
        a.to_lowercase().partial_cmp(&b.to_lowercase()).unwrap()
    });

    if let Some(last) = params.last {
        tags = if let Some(pos) = tags.iter().position(|x| x == &last) {
            tags.split_off(pos+1)
        } else {
            Vec::new()
        }
    }

    if let Some(n) = params.n {
        tags = tags.into_iter().take(n).collect();
    }

    let payload = json!({
        "name": name,
        "tags": tags,
    });

    info!("Tags for \"{}\" found and served", name);
    Json(payload).into_response()
}

async fn get_manifest(
    extract::State(reg): extract::State<SafeReg>,
    extract::Path((name, reference)): extract::Path<(String, String)>,
    range: Option<TypedHeader<Range>>,
) -> impl IntoResponse
{
    let registry = reg.read().await;
    let manifest = registry.get_manifest(&name, &reference).await;

    let Some(payload) = manifest else {
        return NOT_FOUND.into_response();
    };

    info!(
        "Manifest \"{}\" for \"{}\" found and served",
        reference, name
    );
    serve_file(payload, range)
}

async fn head_manifest(
    extract::State(reg): extract::State<SafeReg>,
    extract::Path((name, reference)): extract::Path<(String, String)>,
) -> impl IntoResponse
{
    let registry = reg.read().await;
    let manifest = registry.get_manifest(&name, &reference).await;

    if manifest.is_none() {
        return NOT_FOUND.into_response();
    };

    let msg = format!("Manifest \"{}\" for \"{}\" found", reference, name);
    info!("{}", msg);
    msg.into_response()
}

async fn get_blob(
    extract::State(reg): extract::State<SafeReg>,
    extract::Path((name, digest)): extract::Path<(String, String)>,
    range: Option<TypedHeader<Range>>,
) -> impl IntoResponse
{
    let registry = reg.read().await;
    let blob = registry.get_blob(&name, &digest).await;

    let Some(payload) = blob else {
        return NOT_FOUND.into_response();
    };

    info!("Blob \"{}\" for \"{}\" found and served", digest, name);
    serve_file(payload, range)
}

async fn head_blob(
    extract::State(reg): extract::State<SafeReg>,
    extract::Path((name, digest)): extract::Path<(String, String)>,
) -> impl IntoResponse
{
    let registry = reg.read().await;
    let blob = registry.get_blob(&name, &digest).await;

    if blob.is_none() {
        return NOT_FOUND.into_response();
    };

    let msg = format!("Blob \"{}\" for \"{}\" found", digest, name);
    info!("{}", msg);
    msg.into_response()
}
