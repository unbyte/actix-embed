use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::rc::Rc;

use actix_web::body::BoxBody;
use actix_web::dev::{
    AppService, HttpServiceFactory, ResourceDef, Service, ServiceFactory, ServiceRequest,
    ServiceResponse,
};
use actix_web::http::{header, Method};
use actix_web::HttpResponse;
use futures_core::future::LocalBoxFuture;
use mime_guess::MimeGuess;

use crate::fallback_handler::{DefaultFallbackHandler, FallbackHandler};

/// Wrapper of rust_embed for actix.
///
/// `Embed` service must be registered with `App::service()` method.
///
/// rust_embed documentation: https://docs.rs/rust-embed/
///
/// # Examples
/// ```
/// use actix_web::App;
/// use actix_embed::Embed;
/// use rust_embed::RustEmbed;
///
/// #[derive(RustEmbed)]
/// #[folder = "testdata/"]
/// struct Assets;
///
/// let app = App::new()
///     .service(Embed::new("/static", &Assets));
/// ```
pub struct Embed<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    mount_path: String,
    index_file_path: Option<String>,
    strict_slash: bool,
    fallback_handler: F,
    _f: PhantomData<E>,
}

impl<E, F> Debug for Embed<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Embed")
    }
}

impl<E> Embed<E, DefaultFallbackHandler>
where
    E: 'static + rust_embed::RustEmbed,
{
    /// Create new [Embed] instance.
    ///
    /// # Arguments
    /// The first argument (`mount_path`) is the root URL at which the embed files are served.
    /// For example, `/assets` will serve files at `example.com/assets/...`.
    ///
    /// The second argument (`assets`) is the instance implements [rust_embed::RustEmbed].
    /// For more information, see rust_embed documentation: https://docs.rs/rust-embed/
    ///
    /// # Notes
    /// If the mount path is set as the root path `/`, services registered after this one will
    /// be inaccessible. Register more specific handlers and services before it.
    #[allow(unused_variables)]
    pub fn new<P: AsRef<str>>(mount_path: P, assets: &E) -> Self {
        Embed {
            mount_path: mount_path.as_ref().trim_end_matches('/').to_owned(),
            index_file_path: None,
            strict_slash: false,
            fallback_handler: DefaultFallbackHandler,
            _f: Default::default(),
        }
    }
}

impl<E, F> Embed<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    /// Set whether to ignore the trailing slash of the requested path.
    ///
    /// Defaults to `false`.
    ///
    /// If it's set to true, then file '/dir/file' cannot be accessed by request path '/dir/file/'.
    pub fn strict_slash(mut self, strict_slash: bool) -> Self {
        self.strict_slash = strict_slash;
        self
    }

    /// Set the path of the index file.
    ///
    /// By default there is no index file.
    ///
    /// The index file is treated as the default file returned when a request
    /// visit the root directory.
    pub fn index_file<P: AsRef<str>>(mut self, path: P) -> Self {
        self.index_file_path = Some(
            path.as_ref()
                .trim_end_matches('/')
                .trim_start_matches('/')
                .to_string(),
        );
        self
    }

    /// Sets fallback handler which is used when no matched file could be found.
    ///
    /// The default fallback handler returns 404 responses.
    ///
    /// # Examples
    /// ```
    /// use actix_embed::Embed;
    /// use actix_web::HttpResponse;
    /// use rust_embed::RustEmbed;
    ///
    /// #[derive(RustEmbed)]
    /// #[folder = "testdata/"]
    /// struct Assets;
    ///
    /// # fn run() {
    /// let embed = Embed::new("/static", &Assets)
    ///     .index_file("index.html")
    ///     .fallback_handler(|_: &_| HttpResponse::BadRequest().body("not found"));
    /// # }
    /// ```
    ///
    /// # Note
    /// It is necessary to add type annotation for the closure parameters like `|_: &_| ...`.
    ///
    /// See https://github.com/rust-lang/rust/issues/41078
    pub fn fallback_handler<NF>(self, handler: NF) -> Embed<E, NF>
    where
        NF: FallbackHandler,
    {
        Embed {
            mount_path: self.mount_path,
            index_file_path: self.index_file_path,
            strict_slash: self.strict_slash,
            fallback_handler: handler,
            _f: Default::default(),
        }
    }
}

impl<E, F> HttpServiceFactory for Embed<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    fn register(self, config: &mut AppService) {
        let resource_def = if config.is_root() {
            ResourceDef::root_prefix(&self.mount_path)
        } else {
            ResourceDef::prefix(&self.mount_path)
        };
        config.register_service(resource_def, None, self, None)
    }
}

impl<E, F> ServiceFactory<ServiceRequest> for Embed<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Config = ();
    type Service = EmbedService<E, F>;
    type InitError = ();
    type Future = LocalBoxFuture<'static, Result<Self::Service, Self::InitError>>;

    fn new_service(&self, _: ()) -> Self::Future {
        let strict_slash = self.strict_slash;
        let fallback_handler = self.fallback_handler.clone();
        let index_file_path = self.index_file_path.clone();

        Box::pin(async move {
            Ok(EmbedService::new(EmbedServiceInner {
                strict_slash,
                index_file_path,
                fallback_handler,
            }))
        })
    }
}

#[derive(Clone)]
pub struct EmbedService<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    inner: Rc<EmbedServiceInner<F>>,
    _e: PhantomData<E>,
}

impl<E, F> Debug for EmbedService<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("EmbedService")
    }
}

impl<E, F> EmbedService<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    pub(crate) fn new(inner: EmbedServiceInner<F>) -> Self {
        Self {
            inner: Rc::new(inner),
            _e: Default::default(),
        }
    }
}

pub(crate) struct EmbedServiceInner<F>
where
    F: FallbackHandler,
{
    strict_slash: bool,
    index_file_path: Option<String>,
    fallback_handler: F,
}

impl<E, F> Service<ServiceRequest> for EmbedService<E, F>
where
    E: 'static + rust_embed::RustEmbed,
    F: FallbackHandler,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::always_ready!();

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let this = self.inner.clone();

        Box::pin(async move {
            if Method::GET.ne(req.method()) {
                return Ok(req.into_response(HttpResponse::MethodNotAllowed()));
            }
            let mut path = req.path();
            path = path.trim_start_matches('/');
            if !this.strict_slash {
                path = path.trim_end_matches('/');
            }
            if path.is_empty() {
                path = this.index_file_path.as_deref().unwrap_or("")
            }

            match E::get(path) {
                Some(f) => {
                    let hash = hex::encode(f.metadata.sha256_hash());

                    if req
                        .headers()
                        .get(header::IF_NONE_MATCH)
                        .map(|v| v.to_str().unwrap_or("0").eq(&hash))
                        .unwrap_or(false)
                    {
                        return Ok(req.into_response(HttpResponse::NotModified()));
                    }

                    let mime = MimeGuess::from_path(path).first_or_octet_stream();
                    let data = f.data.into_owned();

                    Ok(req.into_response(
                        HttpResponse::Ok()
                            .content_type(mime.as_ref())
                            .insert_header((header::ETAG, hash))
                            .body(data),
                    ))
                }
                None => {
                    let (req, _) = req.into_parts();
                    let resp = this.fallback_handler.execute(&req);
                    Ok(ServiceResponse::new(req, resp))
                }
            }
        })
    }
}
