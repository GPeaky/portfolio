use ahash::AHashMap;
use mimalloc::MiMalloc;
use ntex::web::{self, App, HttpRequest, HttpResponse};
use parking_lot::RwLock;
use std::{
    fs::{self, File},
    io::{self, Read},
    path::Path,
    sync::Arc,
};
use tokio::time::Instant;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(Clone)]
struct StaticFilesCache {
    cache: Arc<RwLock<AHashMap<String, &'static [u8]>>>,
}

impl StaticFilesCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(AHashMap::new())),
        }
    }

    pub fn initialize(&self) {
        let mut cache = self.cache.write();
        let directory = Path::new("./dist");

        fs::read_dir(directory)
            .expect("Failed to read directory")
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .for_each(|entry| {
                let path = entry.path();
                let contents = Self::read_file(&path);
                let key = Self::generate_cache_key(&path, directory);

                cache.insert(key, contents);
            });
    }

    #[inline]
    pub fn get_file(&self, key: &str) -> Option<&'static [u8]> {
        let cache = self.cache.read();
        cache.get(key).copied()
    }

    fn read_file(path: &Path) -> &'static [u8] {
        let mut contents = Vec::new();
        File::open(path)
            .expect("Failed to open file")
            .read_to_end(&mut contents)
            .expect("Failed to read file");

        let leaked_contents = Box::leak(contents.into_boxed_slice());
        leaked_contents
    }

    fn generate_cache_key(file_path: &Path, base_path: &Path) -> String {
        file_path
            .strip_prefix(base_path)
            .unwrap()
            .to_str()
            .unwrap()
            .replace('\\', "/")
    }
}

async fn serve_file(req: HttpRequest, cache: StaticFilesCache) -> Result<HttpResponse, web::Error> {
    let time = Instant::now();
    let mut path = req.path().trim_start_matches('/');

    if path.is_empty() {
        path = "index.html";
    }

    let response = if let Some(contents) = cache.get_file(path) {
        println!("{} served in {:?}", path, time.elapsed());
        HttpResponse::Ok().content_type("text/html").body(contents)
    } else {
        println!("{} not found, served in {:?}", path, time.elapsed());
        HttpResponse::NotFound().finish()
    };

    Ok(response)
}

#[ntex::main]
async fn main() -> io::Result<()> {
    let cache = StaticFilesCache::new();
    cache.initialize();

    web::server(move || {
        App::new().service(web::resource("/{_:.*}").to({
            let cache = cache.clone();
            move |req| serve_file(req, cache.clone())
        }))
    })
    .bind("0.0.0.0:5173")?
    .run()
    .await
}
