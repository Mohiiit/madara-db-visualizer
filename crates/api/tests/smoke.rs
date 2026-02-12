use axum::body::{to_bytes, Body};
use axum::http::Request;
use db_reader::DbReader;
use indexer::Indexer;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[tokio::test]
async fn health_endpoint_works() {
    let sample = Path::new("./sample-db");
    if !sample.exists() {
        eprintln!("skipping: ./sample-db not present");
        return;
    }

    let db = DbReader::open(sample).expect("open sample db");
    let idx = Indexer::open(":memory:").expect("open in-memory index");

    let state = Arc::new(api::AppState {
        db,
        indexer: Mutex::new(idx),
    });

    let app = api::build_router(state, None);

    let res = app
        .oneshot(
            Request::builder()
                .uri("/api/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(res.status().is_success());

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let s = String::from_utf8_lossy(&body);
    assert!(s.contains("ok"));
}

#[cfg(feature = "embedded-ui")]
#[tokio::test]
async fn embedded_index_html_is_served() {
    use axum::http::Uri;

    let uri: Uri = "/".parse().unwrap();
    let res = api::embedded::response_for_uri(&uri);
    assert!(res.status().is_success());

    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let s = String::from_utf8_lossy(&body);
    assert!(s.contains("Madara DB Visualizer"));
}
