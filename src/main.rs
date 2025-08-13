// Declare .rs files as the module crate::files
mod args;
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

    Ok(())
}
