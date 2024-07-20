use cache::Cache;
use mimalloc::MiMalloc;
use ntex::{
    http::header::{HeaderValue, CONTENT_ENCODING},
    web::{self, App, HttpRequest, HttpResponse},
};
use once_cell::sync::Lazy;

mod cache;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

static CACHE: Lazy<Cache> = Lazy::new(|| {
    println!("Loading files into cache");

    let mut cache = Cache::new();
    cache.initialize("./dist");

    println!("Files loaded & saved in cache");
    cache
});

async fn cached_files(req: HttpRequest) -> HttpResponse {
    let path = req.path();

    if let Some(file) = CACHE.get(path) {
        let mut response = HttpResponse::Ok()
            .content_type(&file.content_type)
            .body(&file.data[..]);

        if file.is_compressed {
            response
                .headers_mut()
                .insert(CONTENT_ENCODING, HeaderValue::from_static("br"));
        }

        response
    } else {
        HttpResponse::NotFound().finish()
    }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    println!("Initializing web server");

    web::server(move || App::new().default_service(web::route().to(cached_files)))
        .bind("0.0.0.0:5174")?
        .run()
        .await
}
