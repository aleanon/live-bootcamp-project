// use axum::{
//     body::Body,
//     extract::State,
//     http::{HeaderValue, Method, Request, StatusCode},
//     middleware::Next,
//     response::Response,
// };

// use crate::app_state::AppState;

// pub async fn dynamic_cors(
//     State(app_state): State<AppState>,
//     req: Request<Body>,
//     next: Next,
// ) -> Response {
//     let origin = req.headers().get("origin").cloned();
//     let is_preflight = req.method() == Method::OPTIONS;

//     if is_preflight {
//         if let Some(origin) = &origin {
//             let origins = &app_state.config.read().await.allowed_origins;
//             if origins.iter().any(|o| *o == origin) {
//                 return Response::builder()
//                     .status(StatusCode::OK)
//                     .header("access-control-allow-origin", origin.clone())
//                     .header("access-control-allow-credentials", "true")
//                     .header(
//                         "access-control-allow-methods",
//                         "GET, POST, PUT, DELETE, OPTIONS",
//                     )
//                     .header(
//                         "access-control-allow-headers",
//                         "content-type, authorization",
//                     )
//                     .header("access-control-max-age", "3600")
//                     .body(Body::empty())
//                     .unwrap();
//             }
//         }
//         return Response::builder()
//             .status(StatusCode::OK)
//             .body(Body::empty())
//             .unwrap();
//     }

//     let res = next.run(req).await;

//     if let Some(origin) = origin {
//         let origins = &app_state.config.read().await.allowed_origins;
//         if origins.iter().any(|o| *o == &origin) {
//             let mut res = res;
//             let headers = res.headers_mut();
//             headers.insert("access-control-allow-origin", origin);
//             headers.insert(
//                 "access-control-allow-credentials",
//                 HeaderValue::from_static("true"),
//             );
//             return res;
//         }
//     }

//     res
// }
