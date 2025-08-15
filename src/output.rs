use crate::{grouping::Group, types::PipelineResult};
use serde::Serialize;

#[derive(Serialize)]
struct JsonGroup {
    avg_distance_bits: f64,
    files: Vec<JsonFile>,
}

#[derive(Serialize)]
struct JsonFile {
    path: String,
    blake3: String,
    dist_bits: u32,
}

pub fn print(groups: &[Group], items: &[PipelineResult], json: bool) {
    match json {
        true => print_json(groups, items),
        false => print_pretty(groups, items),
    }
}

fn print_pretty(groups: &[Group], items: &[PipelineResult]) {
    if groups.is_empty() {
        println!("No likely duplicates found.");
    }

    for (idx, g) in groups.iter().enumerate() {
        println!(
            "Group {} ({} files) - avg dist: {:.2} bits",
            idx + 1,
            g.members.len(),
            g.avg_dist_bits
        );
        for m in &g.members {
            let pr = &items[m.index];
            println!("  - {} (dist: {} bits)", pr.path.display(), m.dist_bits);
        }
    }
}

fn print_json(groups: &[Group], items: &[PipelineResult]) {
    let payload: Vec<JsonGroup> = groups
        .iter()
        .map(|g| JsonGroup {
            avg_distance_bits: g.avg_dist_bits,
            files: g
                .members
                .iter()
                .map(|m| {
                    let pr = &items[m.index];
                    JsonFile {
                        path: pr.path.display().to_string(),
                        blake3: pr.blake3.clone(),
                        dist_bits: m.dist_bits,
                    }
                })
                .collect(),
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
}
