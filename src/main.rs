// Declare .rs files as the module crate::files
mod args;
mod cache;
mod errors;
mod hashing;
mod image_pipeline;
mod progress;
mod scan;
mod types;

use crate::args::Args;
use crate::scan::scan_files;
use clap::Parser;
use std::path::PathBuf;
/// REMOVE Pathbuf ONCE I HAVE TYPE FOR RETURN

fn main() -> Result<(), errors::AppError> {
    let args = Args::parse();

    // Scan
    let media_paths = scan_files(&args.directory, types::DEFAULT_EXTENSIONS);
    println!(
        "Found {} file(s) under \"{}\"",
        media_paths.len(),
        args.directory.display()
    );
    if media_paths.is_empty() {
        return Ok(());
    }

    // Build AppConfig
    let app_cfg = types::AppConfig {
        media_paths,
        hash_alg: args.hash_alg,
        hash_w: args.hash_w,
        hash_h: args.hash_h,
        threshold: args.threshold,
        parallelism: args.parallel,
    };

    // Cache
    let cache_path = std::env::current_exe()?.with_file_name(&args.cache_file);
    let mut cache = cache::load_cache(&cache_path)?;

    // Run Image Pipeline (mutates `cache` in place)
    let image_pipeline_result: Vec<image_pipeline::ImagePipelineResult> =
        image_pipeline::run(app_cfg, &mut cache)?;

    println!("{:#?}", image_pipeline_result);

    // Save Cache
    cache::save_cache(&cache_path, &cache)?;

    Ok(())
}
