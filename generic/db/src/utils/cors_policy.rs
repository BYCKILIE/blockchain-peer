use http::Method;
use tower_http::cors::{Any, CorsLayer};

pub struct CORS;

impl CORS {
    pub fn new() -> CorsLayer {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers(Any)
    }
}
