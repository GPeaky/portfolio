use std::{fs, io::Write, path::Path};

use ahash::AHashMap;
use brotli::{enc::BrotliEncoderParams, CompressorWriter};
use mime_guess::from_path;
use ntex::{http::header::HeaderValue, util::Bytes};

#[derive(Clone)]
pub struct Cache {
    cache: &'static AHashMap<String, FileInfo>,
}

pub struct FileInfo {
    pub content_type: Bytes,
    pub is_compressed: bool,
    pub data: &'static [u8],
}

impl Cache {
    pub fn new(root_path: &str) -> Cache {
        let mut map = AHashMap::new();

        let root_dir = Path::new(root_path);
        let root_len = root_dir.to_str().unwrap().len();
        Cache::load_files_from_dir(&mut map, root_dir, root_len);

        Cache {
            cache: Box::leak(Box::new(map)),
        }
    }

    #[inline]
    pub fn get(&self, key: &str) -> Option<&FileInfo> {
        let cache_key = if !self.cache.contains_key(key) {
            "/index.html"
        } else {
            key
        };

        self.cache.get(cache_key)
    }

    #[inline]
    fn load_files_from_dir(map: &mut AHashMap<String, FileInfo>, dir: &Path, root_len: usize) {
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path: std::path::PathBuf = entry.path();
                    if path.is_dir() {
                        Cache::load_files_from_dir(map, &path, root_len);
                    } else {
                        Cache::insert_file(map, &path, root_len);
                    }
                }
            }
        }
    }

    #[inline]
    fn insert_file(map: &mut AHashMap<String, FileInfo>, path: &Path, root_len: usize) {
        if let Ok(data) = fs::read(path) {
            let mime_type = from_path(path).first_or_octet_stream().to_string();
            let content_type = HeaderValue::from_str(&mime_type).unwrap();
            let should_compress = Cache::should_compress(&mime_type);

            let data = if should_compress {
                Cache::compress_data(&data)
            } else {
                Box::leak(data.into_boxed_slice())
            };

            let key = Cache::generate_key(path, root_len);

            map.insert(
                key,
                FileInfo {
                    content_type: Bytes::copy_from_slice(content_type.as_bytes()),
                    data,
                    is_compressed: should_compress,
                },
            );
        }
    }

    #[inline]
    fn should_compress(mime_type: &str) -> bool {
        matches!(
            mime_type,
            "text/html"
                | "text/css"
                | "application/javascript"
                | "application/json"
                | "image/svg+xml"
        )
    }

    #[inline]
    fn compress_data(data: &[u8]) -> &'static [u8] {
        let mut compressed_data = Vec::new();
        let params = BrotliEncoderParams::default();

        {
            let mut writer = CompressorWriter::with_params(&mut compressed_data, 4096, &params);

            writer.write_all(data).unwrap();
            writer.flush().unwrap();
        }

        Box::leak(compressed_data.into_boxed_slice())
    }

    #[inline]
    fn generate_key(path: &Path, root_len: usize) -> String {
        let key = path.to_str().unwrap().to_string().replace('\\', "/");
        key[root_len..].to_string()
    }
}
