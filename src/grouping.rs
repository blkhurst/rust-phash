use crate::{progress, types::PipelineResult};

#[derive(Debug, Clone)]
pub struct Group {
    pub members: Vec<GroupMember>,
    pub avg_dist_bits: f64,
}

#[derive(Debug, Clone)]
pub struct GroupMember {
    pub index: usize,
    pub dist_bits: u32,
}

/// Group items whose perceptual-hash Hamming distance <= `threshold` (bits).
/// Greedy O(n²). Returns only groups with >1 member.
/// - Members in each group are sorted by ascending distance (most similar first)
/// - Groups are sorted by size desc, then by avg distance asc
pub fn group_duplicates(items: &[PipelineResult], threshold: u32) -> Vec<Group> {
    let n = items.len();
    if n < 2 {
        return Vec::new();
    }

    // Progress Start
    let total_pairs = (n as u64) * ((n.saturating_sub(1)) as u64) / 2;
    let comparing_pb = progress::bar(total_pairs as u64, "Comparing");

    // Decode Image pHash
    let decoded: Vec<img_hash::ImageHash> = items
        .iter()
        .map(|r| img_hash::ImageHash::from_base64(&r.perceptual_hash).expect("valid base64 pHash"))
        .collect();

    let mut visited = vec![false; n];
    let mut groups: Vec<Group> = Vec::new();

    for i in 0..n {
        // Skip if visited
        if visited[i] {
            continue;
        }
        visited[i] = true;

        // Create new Group
        let mut members = vec![GroupMember {
            index: i,
            dist_bits: 0,
        }];

        // Avoid duplicate and reverse comparisons by
        // comparing the current item i with every later item j
        for j in (i + 1)..n {
            comparing_pb.inc(1);

            // Skip if visited
            if visited[j] {
                continue;
            }

            // Hamming distance in bits between two perceptual hashes
            let dist = decoded[i].dist(&decoded[j]);

            // Compare against threshold
            if dist <= threshold {
                visited[j] = true;
                members.push(GroupMember {
                    index: j,
                    dist_bits: dist,
                });
            }
        }

        // Store duplicates and sortBy dist
        if members.len() > 1 {
            members.sort_by_key(|m| m.dist_bits);
            let avg = avg_dist(&members);
            groups.push(Group {
                members,
                avg_dist_bits: avg,
            });
        }
    }

    // Clear Progress Bar
    comparing_pb.finish_and_clear();

    // Sort Groups By Avg
    groups.sort_by(|a, b| a.avg_dist_bits.partial_cmp(&b.avg_dist_bits).unwrap());

    // Sort Groups By Size Then AvgDist
    // groups.sort_by(|a, b| {
    //     b.members
    //         .len()
    //         .cmp(&a.members.len())
    //         .then_with(|| a.avg_dist_bits.partial_cmp(&b.avg_dist_bits).unwrap())
    // });

    groups
}

// Calculate average hamming distance, excluding the first members 0
pub fn avg_dist(members: &[GroupMember]) -> f64 {
    if members.len() <= 1 {
        return 0.0;
    }
    let sum: u128 = members.iter().skip(1).map(|m| m.dist_bits as u128).sum();
    (sum as f64) / ((members.len() - 1) as f64)
}
