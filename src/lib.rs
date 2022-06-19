//! # actix-embed
//!
//! Serve embedded file with actix.
//!
//! ```
//! use actix_web::App;
//! use actix_embed::Embed;
//! use rust_embed::RustEmbed;
//!
//! #[derive(RustEmbed)]
//! #[folder = "testdata/"]
//! struct Assets;
//!
//! let app = App::new()
//!     .service(Embed::new("/static", &Assets));
//! ```
#![warn(missing_docs, missing_debug_implementations)]
#![allow(dead_code)]

pub use fallback_handler::{DefaultFallbackHandler, FallbackHandler};
pub use service::Embed;

mod fallback_handler;
mod service;

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use actix_web::{test, App, HttpResponse};
    use bytes::Bytes;
    use rust_embed::RustEmbed;

    use crate::Embed;

    #[derive(RustEmbed)]
    #[folder = "testdata/"]
    struct Assets;

    #[actix_web::test]
    async fn test_basic() {
        let srv = test::init_service(App::new().service(Embed::new("/", &Assets))).await;

        let cases = [
            ("/index.html", StatusCode::OK),
            ("/assets/index.css", StatusCode::OK),
            ("/assets/index.js", StatusCode::NOT_FOUND),
        ];

        for (path, status) in cases {
            let req = TestRequest::get().uri(path).to_request();
            let resp = test::call_service(&srv, req).await;
            assert_eq!(resp.status(), status);
        }
    }

    #[actix_web::test]
    async fn test_fallback() {
        let srv = test::init_service(App::new().service(
            Embed::new("/", &Assets).fallback_handler(|_: &_| HttpResponse::Ok().body("not found")),
        ))
        .await;

        let req = TestRequest::get().uri("/index.js").to_request();
        let resp = test::call_service(&srv, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        assert_eq!(body, Bytes::from("not found"));
    }

    #[actix_web::test]
    async fn test_strict_slash() {
        // not strict
        {
            let srv = test::init_service(
                App::new().service(Embed::new("/", &Assets).strict_slash(false)),
            )
            .await;

            let req = TestRequest::get().uri("/index.html/").to_request();
            let resp = test::call_service(&srv, req).await;
            assert_eq!(resp.status(), StatusCode::OK);
        }

        // strict
        {
            let srv =
                test::init_service(App::new().service(Embed::new("/", &Assets).strict_slash(true)))
                    .await;

            let req = TestRequest::get().uri("/index.html/").to_request();
            let resp = test::call_service(&srv, req).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }
    }

    #[actix_web::test]
    async fn test_index_file() {
        // not index file
        {
            let srv = test::init_service(App::new().service(Embed::new("/", &Assets))).await;

            let req = TestRequest::get().uri("/").to_request();
            let resp = test::call_service(&srv, req).await;
            assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        }

        // has index file
        {
            let srv = test::init_service(
                App::new().service(Embed::new("/", &Assets).index_file("/index.html")),
            )
            .await;

            let req = TestRequest::get().uri("/").to_request();
            let resp_a = test::call_service(&srv, req).await;
            assert_eq!(resp_a.status(), StatusCode::OK);

            let req = TestRequest::get().uri("/index.html").to_request();
            let resp_b = test::call_service(&srv, req).await;
            assert_eq!(resp_b.status(), StatusCode::OK);

            assert_eq!(test::read_body(resp_a).await, test::read_body(resp_b).await);
        }
    }
}
