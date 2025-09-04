use axum::{
    body::Body,
    extract::State,
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};

use crate::app_state::AppState;

pub async fn dynamic_cors(
    State(app_state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let origin = req.headers().get("origin").cloned();

    let res = next.run(req).await;

    if let Some(origin) = origin {
        let origins = &app_state.config.read().await.0.allowed_origins;
        if origins.iter().any(|o| o == &origin) {
            let mut res = res;
            let headers = res.headers_mut();
            headers.insert("access-control-allow-origin", origin);
            headers.insert(
                "access-control-allow-credentials",
                HeaderValue::from_static("true"),
            );
            return res;
        }
    }

    res
}
