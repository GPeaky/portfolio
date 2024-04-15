use mimalloc::MiMalloc;
use ntex::web::{self, App, HttpRequest, HttpResponse};
use once_cell::sync::OnceCell;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read},
    path::{Path, PathBuf},
};
use tokio::time::Instant;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

static STATIC_FILES: OnceCell<HashMap<String, &'static [u8]>> = OnceCell::new();

fn initialize_static_files_cache() {
    let mut map = HashMap::new();
    let directory = Path::new("./dist");

    fs::read_dir(directory)
        .expect("Failed to read directory")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().is_file())
        .for_each(|entry| {
            let path = entry.path();
            let contents = read_file(&path);
            let key = generate_cache_key(&path, directory);

            map.insert(key, contents);
        });

    STATIC_FILES
        .set(map)
        .expect("Failed to set static files map");
}

fn read_file(path: &PathBuf) -> &'static [u8] {
    let mut contents = Vec::new();
    File::open(path)
        .expect("Failed to open file")
        .read_to_end(&mut contents)
        .expect("Failed to read file");

    Box::leak(contents.into_boxed_slice())
}

fn generate_cache_key(file_path: &Path, base_path: &Path) -> String {
    file_path
        .strip_prefix(base_path)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

async fn serve_file(req: HttpRequest) -> Result<HttpResponse, web::Error> {
    let time = Instant::now();
    let file_name = match Path::new(req.path())
        .file_name()
        .and_then(|name| name.to_str())
    {
        Some("") | None => "index.html",
        Some(name) => name,
    };

    let files_map = STATIC_FILES.get().expect("Static files not initialized");

    let response = if let Some(contents) = files_map.get(file_name) {
        println!("{} served in {:?}", file_name, time.elapsed());
        HttpResponse::Ok().content_type("text/html").body(*contents)
    } else {
        println!("{} not found, served in {:?}", file_name, time.elapsed());
        HttpResponse::NotFound().finish()
    };

    Ok(response)
}

#[ntex::main]
async fn main() -> io::Result<()> {
    initialize_static_files_cache();

    web::server(|| App::new().service(web::resource("/{_:.*}").to(serve_file)))
        .bind("0.0.0.0:5173")?
        .run()
        .await
}
