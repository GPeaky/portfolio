use cache::Cache;
use ntex::{
    http::header::{HeaderValue, CONTENT_ENCODING},
    util::Bytes,
    web::{self, types::State, App, HttpRequest, HttpResponse},
};
use tracing::{info, Level};

mod cache;
mod cache_init;
mod compression;
mod startup_log;

#[cfg(not(test))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[inline(always)]
async fn cached_files(req: HttpRequest, cache: State<Cache>) -> HttpResponse {
    let path = req.path();

    if let Some((file, compressed)) = cache.get(path) {
        let mut response = HttpResponse::Ok()
            .content_type(unsafe {
                HeaderValue::from_shared_unchecked(Bytes::from_static(file.content_type))
            })
            .body(file.data);

        if compressed {
            response
                .headers_mut()
                .insert(CONTENT_ENCODING, HeaderValue::from_static("br"));
        }
        response
    } else {
        HttpResponse::NotFound().finish()
    }
}

// TODO: Load host from .env
#[ntex::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .compact()
        .init();

    info!("🌐 Initializing file cache...");
    let cache = Cache::new("./dist").await;

    info!("🚀 Starting web server on 0.0.0.0:5173");

    web::server(move || {
        App::new()
            .default_service(web::route().to(cached_files))
            .state(cache.clone())
    })
    .bind("127.0.0.1:5174")?
    .run()
    .await
}
