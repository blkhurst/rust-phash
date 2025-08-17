//! Video pipeline: decode+sample → hash frames → aggregate → cache → PipelineResult.

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use indicatif::ParallelProgressIterator;
use rayon::prelude::*;

use super::{aggregate, decode};
use crate::errors::{AppError, VideoError};
use crate::{cache, hashing, progress, types};

pub fn run(
    cfg: types::AppConfig,
    cache: &mut types::CacheFile,
) -> Result<Vec<types::PipelineResult>, AppError> {
    // Initialise ffmpeg
    decode::init_ffmpeg()?;

    // Progress Start
    let progress_bar = progress::bar(cfg.media_paths.len() as u64, "Videos");

    // Shared cache
    let cache_arc = Arc::new(Mutex::new(std::mem::take(cache)));

    // Parallelise Video Processing
    let results: Vec<Result<types::PipelineResult, (PathBuf, AppError)>> = {
        if cfg.parallelism > 0 {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(cfg.parallelism)
                .build()?;
            pool.install(|| {
                cfg.media_paths
                    .par_iter()
                    .map(|p| {
                        process_one_video(p.as_path(), &cfg, &cache_arc).map_err(|e| (p.clone(), e))
                    })
                    .progress_with(progress_bar.clone())
                    .collect()
            })
        } else {
            cfg.media_paths
                .par_iter()
                .map(|p| {
                    process_one_video(p.as_path(), &cfg, &cache_arc).map_err(|e| (p.clone(), e))
                })
                .progress_with(progress_bar.clone())
                .collect()
        }
    };

    // Clear Progress
    progress_bar.finish_and_clear();

    // Collect Successes and Failures
    let mut oks: Vec<types::PipelineResult> = Vec::new();
    let mut errs: Vec<(PathBuf, AppError)> = Vec::new();
    for r in results {
        match r {
            Ok(ok) => oks.push(ok),
            Err((p, e)) => errs.push((p, e)),
        }
    }

    // Error Report
    if !errs.is_empty() {
        errs.iter().for_each(|(p, e)| {
            eprintln!("warn: {} -> {}", p.display(), e);
        });
        eprintln!("note: {} file(s) failed", errs.len());
    }

    // Return cache to caller
    let updated = Arc::into_inner(cache_arc)
        .expect("no other Arc clones left")
        .into_inner()
        .unwrap();
    *cache = updated;

    Ok(oks)
}

fn process_one_video(
    path: &Path,
    cfg: &types::AppConfig,
    cache_arc: &Arc<Mutex<types::CacheFile>>,
) -> Result<types::PipelineResult, AppError> {
    // Compute Blake3 Hash - Parallel
    let key = hashing::compute_blake3(path)?;

    // Cache Hit with matching params? Return Early - Single Thread (Read Lock)
    if let Some(entry) = {
        let cm = cache_arc.lock().unwrap();
        cache::lookup(&cm, &key)
    } {
        let ok = entry.hash_alg == cfg.hash_alg
            && entry.hash_w == cfg.hash_w
            && entry.hash_h == cfg.hash_h
            && entry.sample_start == Some(cfg.sample_start)
            && entry.sample_count == Some(cfg.sample_count)
            && entry.sample_window == Some(cfg.sample_window)
            && entry.aggregation == Some(cfg.aggregation);
        if ok {
            return Ok(types::PipelineResult {
                path: path.to_path_buf(),
                blake3: key,
                perceptual_hash: entry.perceptual_hash.clone(),
            });
        }
    }

    // Decode, Sample, Hash Samples
    let hasher = hashing::build_hasher(cfg.hash_alg, cfg.hash_w, cfg.hash_h);
    let frame_hashes = decode::decode_sample_even_window_hash(
        path,
        cfg.sample_start,
        cfg.sample_count,
        cfg.sample_window,
        &hasher,
    )?;

    // Aggregate to one hash
    let video_hash = match cfg.aggregation {
        types::Aggregation::Medoid => aggregate::aggregate_medoid(&frame_hashes),
        types::Aggregation::Majority => aggregate::aggregate_majority_as_real(&frame_hashes),
    }
    .ok_or_else(|| AppError::Video(VideoError::NoSamples))?;
    let phash_b64 = video_hash.to_base64();

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
                sample_start: Some(cfg.sample_start),
                sample_count: Some(cfg.sample_count),
                sample_window: Some(cfg.sample_window),
                aggregation: Some(cfg.aggregation),
            },
        );
    }

    // Return
    Ok(types::PipelineResult {
        path: path.to_path_buf(),
        blake3: key,
        perceptual_hash: phash_b64,
    })
}
