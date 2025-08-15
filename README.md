# Perceptual Hashing

A performant CLI tool to detect near-duplicate images using **perceptual hashing**.

Built in Rust as my first project, with a focus on **clean structure** and **readability**.

## Overview

* **Scanning** - Recursively walk directory, filtering by `DEFAULT_EXTENSIONS`.
* **Hashing** - Computes:
  * Compute **BLAKE3** file hash as identifier (highly parallelisable).
  * Generate **Perceptual hash** (Mean, Gradient, or DoubleGradient) for similarity detection.
* **Caching** - Stores results in a JSON file keyed by content hash; renames/moves don’t trigger recomputation.
* **Parallel Pipeline** - Runs hashing in parallel with `Rayon`, showing progress with `indicatif`.
* **Grouping** - Greedy clustering of images whose perceptual hash Hamming distance is <= threshold.
* **Output** - Outputs results to CLI, with JSON support for processing.
* **Error Handling** - Using `thiserror` for clean, minimal boilerplate error propagation.

```bash
src
├── args.rs             # Argument parsing with `clap`.
├── cache.rs            # Load/save JSON cache.
├── errors.rs           # Centralised error types using `thiserror`.
├── grouping.rs         # Greedy grouping based on Hamming distance.
├── hashing.rs          # Compute BLAKE3 and perceptual hashes.
├── image_pipeline.rs   # Orchestrates hashing & caching in parallel with Rayon.
├── main.rs             # Entry point.
├── output.rs           # Pretty and JSON output for results.
├── progress.rs         # Progress bars using `indicatif`.
├── scan.rs             # Recursive file scanning with extension filtering.
└── types.rs            # Shared types, constants, and config structures.
```

## Usage

```bash
# Build ./targets/release/rust-phash
cargo build --release

# Usage
Usage: rust-phash [OPTIONS] <DIRECTORY> [THRESHOLD]

Arguments:
  <DIRECTORY>  Directory to recursively scan
  [THRESHOLD]  Hamming distance threshold [default: 10]

Options:
      --json                     Print JSON output
      --hash-alg <HASH_ALG>      Hashing Algorithm [default: double-gradient]
                                    [possible values: mean, gradient, double-gradient]
      --hash-w <HASH_W>          Hash width (bits across) [default: 16]
      --hash-h <HASH_H>          Hash height (bits down) [default: 16]
      --parallel <PARALLEL>      Maximum parallelism (Rayon threads) [default: 0 auto]
      --cache-file <CACHE_FILE>  Cache file path [default: .phash-cache.json]
  -h, --help                     Print help
```

**Example JSON Output**
```json
[
  {
    "avg_distance_bits": 5,
    "files": [
      { "path": "img1.jpg", "dist_bits": 0 },
      { "path": "img2.jpg", "dist_bits": 5 }
    ]
  }
]
```

## Notes
- Threshold sensitivity depends on hash dimensions. Changing `hash-w` and `hash-h` alters the total bits, so you may need to adjust the threshold.
