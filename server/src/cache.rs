use std::{fs, io::Write, path::Path};

use ahash::AHashMap;
use brotli::{enc::BrotliEncoderParams, CompressorWriter};
use mime_guess::from_path;

pub struct Cache {
    cache: AHashMap<String, (String, Vec<u8>, bool)>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            cache: AHashMap::new(),
        }
    }

    // Todo - Use parallelism to load files (Not mandatory because is only for loading files at startup)
    pub fn initialize(&mut self, root_path: &str) {
        let root_dir = Path::new(root_path);
        let root_len = root_dir.to_str().unwrap().len();

        self.load_files_from_dir(root_dir, root_len);
    }

    pub fn get(&self, key: &str) -> Option<(&str, &[u8], &bool)> {
        let cache_key = if !self.cache.contains_key(key) {
            "/index.html"
        } else {
            key
        };

        self.cache
            .get(cache_key)
            .map(|(mime_type, data, compressed)| (mime_type.as_str(), data.as_slice(), compressed))
    }

    fn insert_file(&mut self, path: &Path, root_len: usize) {
        if let Ok(data) = fs::read(path) {
            let mime_type = from_path(path).first_or_octet_stream().to_string();
            let should_compress = matches!(
                mime_type.as_str(),
                "text/html"
                    | "text/css"
                    | "application/javascript"
                    | "application/json"
                    | "image/svg+xml"
            );

            let data = if should_compress {
                let params = BrotliEncoderParams::default();
                let mut compressed_data = Vec::new();

                {
                    let mut writter =
                        CompressorWriter::with_params(&mut compressed_data, 4096, &params);
                    writter.write_all(&data).unwrap();
                    writter.flush().unwrap();
                }

                compressed_data
            } else {
                data
            };

            let mut key = path.to_str().unwrap().to_string().replace('\\', "/");
            key = key[root_len..].to_string();

            self.cache.insert(key, (mime_type, data, should_compress));
        }
    }

    fn load_files_from_dir(&mut self, dir: &Path, root_len: usize) {
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_dir() {
                        self.load_files_from_dir(&path, root_len);
                    } else {
                        self.insert_file(&path, root_len);
                    }
                }
            }
        }
    }
}
