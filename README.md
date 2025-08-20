# Perceptual Hashing

A performant CLI tool to detect near-duplicate images and videos using **perceptual hashing**.

## Overview

* **Scanning** - Recursively walk directory, filtering by file extensions.
* **Hashing** - Computes:
  * **BLAKE3** file hash as identifier (highly parallelisable).
  * **Perceptual hash** (Mean, Gradient, or DoubleGradient) for similarity detection.
* **Caching** - Stores results in a JSON file keyed by content hash; renames/moves don’t trigger recomputation.
* **Parallel Pipeline** - Runs hashing in parallel with `Rayon`, showing progress with `indicatif`.
* **Grouping** - Greedy clustering of images whose perceptual hash Hamming distance is <= threshold.
* **Output** - Outputs results to CLI, with JSON support for processing.
* **Error Handling** - Using `thiserror` for clean, minimal boilerplate error propagation.

```bash
src
├── video
│   ├── aggregate.rs    # Medoid / Majority
│   ├── decode.rs       # FFmpeg decode, sample, RGB convert
│   ├── pipeline.rs     # Decode, sample, hash, aggregate, cache
│   └── sample.rs       # Total frame estimation + sampling plan
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
      --json                           Print JSON output
      --hash-alg <HASH_ALG>            Hashing Algorithm [default: double-gradient] [possible values: mean, gradient, double-gradient]
      --hash-w <HASH_W>                Hash width (bits across) [default: 16]
      --hash-h <HASH_H>                Hash height (bits down) [default: 16]
      --parallel <PARALLEL>            Maximum parallelism (Rayon threads) [default: 0]
      --cache-file <CACHE_FILE>        Cache file path [default: .phash-cache.json]
      --video                          Process videos instead of images
      --sample-start <SAMPLE_START>    Video ~ Frame to start sampling from [default: 0]
      --sample-count <SAMPLE_COUNT>    Video ~ Number of frames samples; evenly-spaced between sample-start and sample-window [default: 10]
      --sample-window <SAMPLE_WINDOW>  Video ~ Number of frames to sample over; 0 = auto (whole video) [default: 0]
      --aggregation <AGGREGATION>      Video ~ Aggregation method [default: medoid] [possible values: majority, medoid]
      --output <OUTPUT>                Output JSON to a file
  -h, --help                           Print help
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


## Video
### Process
* **Decode** – Use FFmpeg to open the video and convert frames to RGB24.
* **Sample** – Select frames evenly across the sample window until `sample_count` is reached.
  * `sample_start` - If greater than `total_frames`, set to `total_frames / 2`.
  * `sample_window` - If this would overflow end; the window is shrunk.
* **Hash Frames** – Apply the chosen image perceptual hashing algorithm to each sampled frame.
* **Aggregate** – Combine frame hashes into a single video fingerprint using either:
  * **Majority** – Bitwise majority vote across frames.
  * **Medoid** – Select the frame hash with the lowest total Hamming distance to the others.
* **Cache & Compare** – Store the resulting video hash in the JSON cache (atomic writes prevent corruption). During duplicate detection, compare video fingerprints just like images.

### Requirements
```bash
# Requires ffmpeg
sudo apt install -y ffmpeg

# Build requires FFmpeg development headers
sudo apt install -y pkg-config libavutil-dev libavformat-dev libavcodec-dev libswscale-dev libavfilter-dev libavdevice-dev
```

## Notes
- Threshold sensitivity depends on hash dimensions. Changing `hash-w` and `hash-h` alters the total bits, so you may need to adjust the threshold.
- Denser frame sampling generally improves accuracy, lowering false-positives.
- Video codecs use inter-frame compression: most frames depend on previous ones (reference chain). Decoding deeper into this chain is slower, so smaller `sample_start` and `sample_window` improve efficiency.
