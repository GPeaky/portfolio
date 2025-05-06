use async_compression::tokio::write::BrotliEncoder;
use tokio::io::AsyncWriteExt;

pub struct CompressedData {
    pub data: &'static [u8],
    pub is_compressed: bool,
}

#[inline(always)]
pub async fn compress_if_needed(data: &[u8], mime_type: &str) -> CompressedData {
    if should_compress(mime_type) {
        CompressedData {
            data: compress_brotli(data).await,
            is_compressed: true,
        }
    } else {
        CompressedData {
            data: Box::leak(data.to_vec().into_boxed_slice()),
            is_compressed: false,
        }
    }
}

#[inline(always)]
fn should_compress(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "text/html" | "text/css" | "application/javascript" | "application/json" | "image/svg+xml"
    )
}

async fn compress_brotli(data: &[u8]) -> &'static [u8] {
    let compressed = Vec::with_capacity(data.len());
    let mut encoder = BrotliEncoder::with_quality(compressed, async_compression::Level::Best);

    encoder.write_all(data).await.unwrap();
    encoder.shutdown().await.unwrap();

    Box::leak(encoder.into_inner().into_boxed_slice())
}
