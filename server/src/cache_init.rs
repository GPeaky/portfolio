use ahash::AHashMap;
use mime_guess::from_path;
use ntex::http::header::HeaderValue;
use ntex::util::join_all;
use std::path::{Path, PathBuf};

use crate::cache::{Cache, FileInfo};
use crate::compression::{CompressedData, compress_if_needed};
use crate::startup_log::{FileLoadStats, print_load_stats};

struct PathsInfo {
    paths: Vec<PathBuf>,
    compressible_count: usize,
    non_compressible_count: usize,
}

impl Cache {
    pub async fn new(root_path: &str) -> Cache {
        let root_dir = Path::new(root_path);
        let root_len = root_dir.to_str().unwrap().len();

        let PathsInfo {
            paths,
            compressible_count,
            non_compressible_count,
        } = collect_paths_info(root_dir).await;

        let mut compressed_files = AHashMap::with_capacity(compressible_count);
        let mut files = AHashMap::with_capacity(non_compressible_count);
        let mut stats = Vec::with_capacity(compressible_count + non_compressible_count);

        let tasks: Vec<_> = paths
            .into_iter()
            .map(|path| process_file(path, root_len))
            .collect();

        let results = join_all(tasks).await;

        for (key, file_info, stat) in results.into_iter().flatten() {
            if stat.is_compressed {
                compressed_files.insert(key, file_info);
            } else {
                files.insert(key, file_info);
            }
            stats.push(stat);
        }

        print_load_stats(&stats);

        Cache {
            compressed_files: Box::leak(Box::new(compressed_files)),
            files: Box::leak(Box::new(files)),
        }
    }
}

async fn collect_paths_info(dir: &Path) -> PathsInfo {
    let mut paths = Vec::new();
    let mut compressible_count = 0;
    let mut non_compressible_count = 0;

    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(_) => {
            return PathsInfo {
                paths,
                compressible_count,
                non_compressible_count,
            };
        }
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            let subdir_info = Box::pin(collect_paths_info(&path)).await;
            paths.extend(subdir_info.paths);
            compressible_count += subdir_info.compressible_count;
            non_compressible_count += subdir_info.non_compressible_count;
        } else {
            if let Some(mime) = from_path(&path).first() {
                if matches!(
                    mime.as_ref(),
                    "text/html"
                        | "text/css"
                        | "application/javascript"
                        | "application/json"
                        | "image/svg+xml"
                ) {
                    compressible_count += 1;
                } else {
                    non_compressible_count += 1;
                }
            } else {
                non_compressible_count += 1;
            }
            paths.push(path);
        }
    }

    PathsInfo {
        paths,
        compressible_count,
        non_compressible_count,
    }
}

async fn process_file(
    path: PathBuf,
    root_len: usize,
) -> Option<(&'static str, FileInfo, FileLoadStats)> {
    let data = tokio::fs::read(&path).await.ok()?;
    let original_size = data.len();
    let mime_type = from_path(&path).first_or_octet_stream().to_string();

    let CompressedData {
        data: compressed_data,
        is_compressed,
    } = compress_if_needed(&data, &mime_type).await;

    let key = unsafe { generate_key(&path, root_len) };
    let file_info = FileInfo {
        content_type: Box::leak(Box::from(
            HeaderValue::from_str(&mime_type).unwrap().as_bytes(),
        )),
        data: compressed_data,
    };

    let stat = FileLoadStats {
        name: path.file_name().unwrap().to_str().unwrap().to_string(),
        original_size,
        final_size: compressed_data.len(),
        is_compressed,
    };

    Some((key, file_info, stat))
}

#[inline(always)]
unsafe fn generate_key(path: &Path, root_len: usize) -> &'static str {
    let path_str = unsafe { path.to_str().unwrap_unchecked() };

    #[cfg(windows)]
    {
        let mut string = String::with_capacity(path_str.len());
        string.extend(
            path_str[root_len..]
                .chars()
                .map(|c| if c == '\\' { '/' } else { c }),
        );
        Box::leak(string.into_boxed_str())
    }

    #[cfg(not(windows))]
    {
        Box::leak(path_str[root_len..].to_string().into_boxed_str())
    }
}
