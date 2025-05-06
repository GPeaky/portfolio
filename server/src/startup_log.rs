use tracing::info;

pub struct FileLoadStats {
    pub name: String,
    pub original_size: usize,
    pub final_size: usize,
    pub is_compressed: bool,
}

// TODO: Prettier prints
pub fn print_load_stats(stats: &[FileLoadStats]) {
    let mut total_original = 0;
    let mut total_final = 0;
    let mut num_compressed = 0;

    info!("File Loading Report:");
    info!("{:-^80}", "");
    info!(
        "{:<30} {:>10} {:>10} {:>10} {:<8}",
        "File", "Original", "Final", "Reduction", "Type"
    );
    info!("{:-^80}", "");

    for stat in stats {
        total_original += stat.original_size;
        total_final += stat.final_size;
        if stat.is_compressed {
            num_compressed += 1;
        }

        info!(
            "{:<30} {:>10} {:>10} {:>9.1}% {:<8}",
            stat.name,
            format!("{:.2}KB", stat.original_size as f64 / 1024.0),
            format!("{:.2}KB", stat.final_size as f64 / 1024.0),
            100.0 * (1.0 - (stat.final_size as f64 / stat.original_size as f64)),
            if stat.is_compressed { "BROTLI" } else { "NONE" }
        );
    }

    info!("{:-^80}", "");
    info!(
        "Total Files: {}, Compressed: {}",
        stats.len(),
        num_compressed
    );
    info!(
        "Total Size: {:.2}MB -> {:.2}MB ({:.1}% reduction)",
        total_original as f64 / (1024.0 * 1024.0),
        total_final as f64 / (1024.0 * 1024.0),
        100.0 * (1.0 - (total_final as f64 / total_original as f64))
    );
}
