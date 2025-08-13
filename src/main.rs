// Declare .rs files as the module crate::files
mod args;
mod cache;
mod errors;
mod hashing;
mod progress;
mod scan;
mod types;

use crate::args::Args;
use crate::hashing::compute_blake3;
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

    // Progress Start
    let mp = progress::multi();
    let hashing_pb = mp.add(progress::bar(media_paths.len() as u64, "Hashing"));

    // Hasher
    let hasher = hashing::build_hasher(hash_alg, hash_w, hash_h);

    // Iterate - Perceptual Hashing
    for p in &media_paths {
        // Increment ProgressBar
        hashing_pb.inc(1);

        // Compute file hash
        let key = compute_blake3(p)?;

        // Cache Hit?
        if cache::lookup(&cache, &key).is_some() {
            continue;
        }

        // Compute perceptual hash
        let perceptual_hash = hashing::perceptual_hash(p, &hasher)?;

        // Update CacheFile
        cache::upsert(
            &mut cache,
            key,
            types::CacheEntry {
                hash_alg: hash_alg,
                hash_w: hash_w,
                hash_h: hash_h,
                perceptual_hash: perceptual_hash,
            },
        );
    }

    // Clear Progress
    hashing_pb.finish_and_clear();

    // Save Cache
    cache::save_cache(&cache_path, &cache)?;

    Ok(())
}
