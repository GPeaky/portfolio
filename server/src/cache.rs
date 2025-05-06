use ahash::AHashMap;

#[derive(Clone)]
pub struct Cache {
    pub(crate) compressed_files: &'static AHashMap<&'static str, FileInfo>,
    pub(crate) files: &'static AHashMap<&'static str, FileInfo>,
}

pub struct FileInfo {
    pub(crate) content_type: &'static [u8],
    pub(crate) data: &'static [u8],
}

impl Cache {
    #[inline(always)]
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
}
