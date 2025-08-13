// Declare .rs files as the module crate::files
mod args;
mod cache;
mod errors;
mod scan;
mod types;

use crate::args::Args;
use crate::scan::scan_files;
use clap::Parser;

fn main() -> Result<(), errors::AppError> {
    let args = Args::parse();

    let directory = args
        .directory
        .canonicalize()
        .unwrap_or(args.directory.clone());
    let threshold = args.threshold;
    let json = args.json;
    let hash_alg = args.hash_alg;
    let hash_w = args.hash_w;
    let hash_h = args.hash_h;
    let parallel = args.parallel;
    let cache_file = args.cache_file;

    // Scan
    let media_paths = scan_files(&directory, types::DEFAULT_EXTENSIONS);
    println!(
        "Found {} file(s) under \"{}\"",
        media_paths.len(),
        directory.display()
    );
    if media_paths.is_empty() {
        println!("Nothing to do.");
        return Ok(());
    }

    // Cache
    let cache_path = std::env::current_exe()?.with_file_name(&cache_file);
    let mut cache = cache::load_cache(&cache_path)?;

    // Iterate
    for p in &media_paths {
        let key = p
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let perceptual_hash = String::new();

        let entry = types::CacheEntry {
            hash_alg: hash_alg,
            hash_w: hash_w,
            hash_h: hash_h,
            perceptual_hash: perceptual_hash,
        };

        cache::upsert(&mut cache, key, entry);

        println!("Debug: {:#?}", p.file_name())
    }

    // Save Cache
    cache::save_cache(&cache_path, &cache)?;

    Ok(())
}
