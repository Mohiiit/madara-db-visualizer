use axum::body::Body;
use axum::http::{header, HeaderMap, HeaderValue, Response, StatusCode, Uri};
use bytes::Bytes;
use include_dir::{include_dir, Dir};

// `include_dir!` requires a string literal (it supports `$CARGO_MANIFEST_DIR`), so we embed the
// workspace-level `dist/` directory after it has been built via `scripts/build_dist.sh`.
static DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../dist");

pub fn response_for_uri(uri: &Uri) -> Response<Body> {
    // Normalize path.
    let mut path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        path = "index.html";
    }

    // Try direct match first.
    if let Some(resp) = file_response(path) {
        return resp;
    }

    // SPA fallback.
    if let Some(resp) = file_response("index.html") {
        return resp;
    }

    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("embedded dist missing index.html"))
        .unwrap()
}

fn file_response(path: &str) -> Option<Response<Body>> {
    let file = DIST.get_file(path)?;
    let contents = file.contents();

    let mime = mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string();

    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_str(&mime).ok()?);

    // Cache policy: index.html no-cache; hashed assets immutable.
    if path == "index.html" {
        headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    } else if looks_hashed_asset(path) {
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );
    }

    let body = Body::from(Bytes::copy_from_slice(contents));
    Some(
        Response::builder()
            .status(StatusCode::OK)
            .body(body)
            .map(|mut r| {
                *r.headers_mut() = headers;
                r
            })
            .ok()?,
    )
}

fn looks_hashed_asset(path: &str) -> bool {
    // Trunk outputs assets like frontend-<hash>.js and frontend-<hash>_bg.wasm.
    (path.starts_with("frontend-") && path.ends_with(".js"))
        || (path.starts_with("frontend-") && path.ends_with(".wasm"))
        || (path.starts_with("frontend-") && path.ends_with("_bg.wasm"))
}
