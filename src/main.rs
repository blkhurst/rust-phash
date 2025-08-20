// Declare .rs files as the module crate::files
mod args;
mod cache;
mod errors;
mod grouping;
mod hashing;
mod image_pipeline;
mod output;
mod progress;
mod scan;
mod types;
mod video;

use crate::args::Args;
use crate::scan::scan_files;
use clap::Parser;

fn main() -> Result<(), errors::AppError> {
    let args = Args::parse();

    // Scan
    let extensions: &[&str] = match args.video {
        true => types::VIDEO_EXTENSIONS,
        false => types::IMAGE_EXTENSIONS,
    };
    let media_paths = scan_files(&args.directory, extensions);
    eprintln!(
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
        parallelism: args.parallel,
        sample_start: args.sample_start,
        sample_count: args.sample_count,
        sample_window: args.sample_window,
        aggregation: args.aggregation,
    };

    // Cache
    let cache_path = std::env::current_exe()?.with_file_name(&args.cache_file);
    let mut cache = cache::load_cache(&cache_path)?;

    // Run Image Pipeline (mutates `cache` in place)
    let pipeline_results: Vec<types::PipelineResult> = match args.video {
        true => video::pipeline::run(app_cfg, &mut cache)?,
        false => image_pipeline::run(app_cfg, &mut cache)?,
    };

    // Group Near Duplicates (Calculate Hamming Distance)
    let groups = grouping::group_duplicates(&pipeline_results, args.threshold);

    // Output or Print
    if let Some(output_path) = &args.output {
        output::write_json_file(&groups, &pipeline_results, output_path)?;
        eprintln!("\nResults written to \"{}\"", output_path.display());
    } else {
        output::print(&groups, &pipeline_results, args.json);
    }

    // Save Cache
    cache::save_cache(&cache_path, &cache)?;

    Ok(())
}
