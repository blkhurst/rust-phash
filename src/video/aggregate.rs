use img_hash::ImageHash;

/// Aggregate frame hashes into a single video hash using medoid (least Hamming distance to all other frames).
pub fn aggregate_medoid(hashes: &[ImageHash]) -> Option<ImageHash> {
    if hashes.is_empty() {
        return None;
    }
    if hashes.len() == 1 {
        return Some(hashes[0].clone());
    }

    let mut best_idx = 0usize;
    let mut best_sum = u64::MAX;

    for (i, hi) in hashes.iter().enumerate() {
        // Uses img_hash ImageHash.dist to calculate hamming distance.
        let sum: u64 = hashes.iter().map(|hj| hi.dist(hj) as u64).sum();
        if sum < best_sum {
            best_sum = sum;
            best_idx = i;
        }
    }
    Some(hashes[best_idx].clone())
}

/// Aggregate frame hashes into one hash by majority:
/// 1) Compute the bitwise-majority hash across all frames (ties→1).
/// 2) Return the *real* frame hash that is closest (smallest Hamming distance) to that majority.
pub fn aggregate_majority_as_real(hashes: &[ImageHash]) -> Option<ImageHash> {
    match hashes {
        [] => return None,
        [only] => return Some(only.clone()),
        _ => {}
    }

    // 1) Majority bitstring across all hashes (ties → 1)
    let majority = majority_bytes(hashes);

    // 2) Pick the real frame hash closest to the majority
    hashes
        .iter()
        .min_by_key(|h| hamming_bytes(h.as_bytes(), &majority))
        .cloned()
}

fn majority_bytes(hashes: &[ImageHash]) -> Vec<u8> {
    // All hashes must have the same length; use the first as reference.
    let len = hashes[0].as_bytes().len();
    let mut counts = vec![0i32; len * 8];

    // For each bit position, add +1 for a '1', -1 for a '0'.
    // After summing all frames, count>=0 means majority '1' (ties→1).
    for h in hashes {
        for (i, &byte) in h.as_bytes().iter().enumerate() {
            let mut b = byte;
            for bit in 0..8 {
                counts[i * 8 + bit] += if (b & 1) != 0 { 1 } else { -1 };
                b >>= 1;
            }
        }
    }

    // Rebuild bytes from bit counts.
    counts
        .chunks_exact(8)
        .map(|chunk| {
            let mut byte = 0u8;
            for (bit, &c) in chunk.iter().enumerate() {
                if c >= 0 {
                    byte |= 1 << bit;
                } // ties -> 1
            }
            byte
        })
        .collect()
}

fn hamming_bytes(a: &[u8], b: &[u8]) -> u32 {
    debug_assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(&x, &y)| (x ^ y).count_ones()).sum()
}
