use std::{fs, io::Write, path::Path};

use ahash::AHashMap;
use brotli::{enc::BrotliEncoderParams, CompressorWriter};
use mime_guess::from_path;
use ntex::{http::header::HeaderValue, util::Bytes};

#[derive(Clone)]
pub struct Cache {
    compressed_files: &'static AHashMap<String, FileInfo>,
    files: &'static AHashMap<String, FileInfo>,
}

pub struct FileInfo {
    pub content_type: Bytes,
    pub data: &'static [u8],
}

impl Cache {
    pub fn new(root_path: &str) -> Cache {
        let mut compressed_files = AHashMap::new();
        let mut files = AHashMap::new();

        let root_dir = Path::new(root_path);
        let root_len = root_dir.to_str().unwrap().len();
        Cache::load_files_from_dir(&mut compressed_files, &mut files, root_dir, root_len);

        Cache {
            compressed_files: Box::leak(Box::new(compressed_files)),
            files: Box::leak(Box::new(files)),
        }
    }

    #[inline]
    pub fn get(&self, key: &str) -> Option<(&FileInfo, bool)> {
        if let Some(file_info) = self.compressed_files.get(key) {
            return Some((file_info, true));
        }

        if let Some(file_info) = self.files.get(key) {
            return Some((file_info, false));
        }

        self.compressed_files
            .get("/index.html")
            .map(|file_info| (file_info, true))
    }

    #[inline]
    fn load_files_from_dir(
        compressed_files: &mut AHashMap<String, FileInfo>,
        files: &mut AHashMap<String, FileInfo>,
        dir: &Path,
        root_len: usize,
    ) {
        if dir.is_dir() {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path: std::path::PathBuf = entry.path();
                    if path.is_dir() {
                        Cache::load_files_from_dir(compressed_files, files, &path, root_len);
                    } else {
                        Cache::insert_file(compressed_files, files, &path, root_len);
                    }
                }
            }
        }
    }

    #[inline]
    fn insert_file(
        compressed_files: &mut AHashMap<String, FileInfo>,
        files: &mut AHashMap<String, FileInfo>,
        path: &Path,
        root_len: usize,
    ) {
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
            let file_info = FileInfo {
                content_type: Bytes::copy_from_slice(content_type.as_bytes()),
                data,
            };

            match should_compress {
                true => {
                    compressed_files.insert(key, file_info);
                }

                false => {
                    files.insert(key, file_info);
                }
            }
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
