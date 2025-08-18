use crate::{cache, errors::AppError, hashing, progress, types};
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

/// Run Image Pipeline in parallel using Rayon
pub fn run(
    cfg: types::AppConfig,
    cache: &mut types::CacheFile,
) -> Result<Vec<types::PipelineResult>, AppError> {
    // Progress Start
    let hashing_pb = progress::bar(cfg.media_paths.len() as u64, "Hashing");

    // Wrap the caller-owned cache in Arc<Mutex<_>> for thread-safe mutation during parallel work.
    let cache_arc = Arc::new(Mutex::new(std::mem::take(cache)));

    // Drive the parallel work with an optional fixed-size pool using your helper.
    let results: Vec<Result<types::PipelineResult, (PathBuf, AppError)>> = {
        if cfg.parallelism > 0 {
            // Configure Rayon
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(cfg.parallelism)
                .build()?;
            // Run
            pool.install(|| {
                cfg.media_paths
                    .par_iter()
                    .progress_with(hashing_pb.clone())
                    .map(|p| process_path(p, &cfg, &cache_arc).map_err(|e| (p.clone(), e)))
                    .collect()
            })
        } else {
            // Let Rayon decide
            cfg.media_paths
                .par_iter()
                .progress_with(hashing_pb.clone())
                .map(|p| process_path(p, &cfg, &cache_arc).map_err(|e| (p.clone(), e)))
                .collect()
        }
    };

    // Collect
    let collected: Vec<types::PipelineResult> =
        results.into_iter().filter_map(|r| r.ok()).collect();

    // Clear Progress
    hashing_pb.finish_and_clear();

    // Write the updated cache back into the caller-owned value (Arc Unwrap).
    let updated = Arc::into_inner(cache_arc)
        .expect("no other Arc clones left")
        .into_inner()
        .unwrap();
    *cache = updated;

    Ok(collected)
}

/// Process a single file (fileHash + perceptualHash). Called by image_pipeline::run
fn process_path(
    p: &Path,
    cfg: &types::AppConfig,
    cache_arc: &Arc<Mutex<types::CacheFile>>,
) -> Result<types::PipelineResult, AppError> {
    // Compute Blake3 Hash - Parallel
    let key = hashing::compute_blake3(p)?;

    // Cache Hit? Return Early - Single Thread (Read Lock)
    {
        let cm = cache_arc.lock().unwrap();
        if let Some(entry) = cache::lookup(&cm, &key, cfg, /*is_video=*/ false) {
            // Return PipelineResult
            return Ok(types::PipelineResult {
                path: p.to_path_buf(),
                blake3: key.clone(),
                perceptual_hash: entry.perceptual_hash.clone(),
            });
        }
    }

    // Compute Perceptual Hash - Parallel
    let hasher = hashing::build_hasher(cfg.hash_alg, cfg.hash_w, cfg.hash_h);
    let phash_b64 = hashing::compute_perceptual_hash(p, &hasher)?;

    // Upsert - Single Thread (Write Lock)
    {
        let mut cm = cache_arc.lock().unwrap();
        cache::upsert(
            &mut cm,
            key.clone(),
            types::CacheEntry {
                hash_alg: cfg.hash_alg,
                hash_w: cfg.hash_w,
                hash_h: cfg.hash_h,
                perceptual_hash: phash_b64.clone(),
                sample_start: None,
                sample_count: None,
                sample_window: None,
                aggregation: None,
            },
        );
    }

    // Return
    Ok(types::PipelineResult {
        path: p.to_path_buf(),
        blake3: key,
        perceptual_hash: phash_b64,
    })
}
