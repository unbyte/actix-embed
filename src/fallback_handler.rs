use actix_web::{HttpRequest, HttpResponse};

/// Fallback handlers will be called when no matched file could be found.
pub trait FallbackHandler: 'static + Clone {
    #[allow(missing_docs)]
    fn execute(&self, req: &HttpRequest) -> HttpResponse;
}

impl<T> FallbackHandler for T
where
    T: Fn(&HttpRequest) -> HttpResponse + Clone + 'static,
{
    fn execute(&self, req: &HttpRequest) -> HttpResponse {
        (self)(req)
    }
}

/// The default fallback handler.
///
/// It returns 404 response regardless request information.
#[derive(Debug, Clone)]
pub struct DefaultFallbackHandler;

impl FallbackHandler for DefaultFallbackHandler {
    fn execute(&self, _: &HttpRequest) -> HttpResponse {
        HttpResponse::NotFound().body("404 Not Found")
    }
}
