//! Append-only hourly stats logging for unhub.
//!
//! One line is written on service boot and once per hour. Each line contains
//! only aggregated counts — no installation_ids or other personal identifiers
//! ever touch disk.
//!
//! Log line format (space-separated key=value, one line):
//! ```text
//! 2026-04-26T15:00:00Z kind=boot players_1h=0 players_24h=0 versions_1h= versions_24h=
//! 2026-04-26T16:00:00Z kind=hourly players_1h=42 players_24h=150 versions_1h=0.4.1:30,0.4.2-beta1:12 versions_24h=0.4.1:100,0.4.2-beta1:50
//! ```

use crate::state::HubState;
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

fn collect_version_counts(cache: &moka::sync::Cache<Uuid, String>) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for (_, version) in cache.iter() {
        *counts.entry(version).or_insert(0) += 1;
    }
    counts
}

fn format_version_map(counts: &HashMap<String, usize>) -> String {
    if counts.is_empty() {
        return String::new();
    }
    // Sort by count descending, then alphabetically for tie-breaking.
    let mut pairs: Vec<(&String, &usize)> = counts.iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    pairs
        .iter()
        .map(|(v, c)| format!("{}:{}", v, c))
        .collect::<Vec<_>>()
        .join(",")
}

fn build_log_line(kind: &str, state: &HubState) -> String {
    state.players_1h.run_pending_tasks();
    state.players_24h.run_pending_tasks();

    let counts_1h = collect_version_counts(&state.players_1h);
    let counts_24h = collect_version_counts(&state.players_24h);

    let players_1h = state.players_1h.entry_count();
    let players_24h = state.players_24h.entry_count();

    let versions_1h = format_version_map(&counts_1h);
    let versions_24h = format_version_map(&counts_24h);

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ");

    format!(
        "{} kind={} players_1h={} players_24h={} versions_1h={} versions_24h={}",
        now, kind, players_1h, players_24h, versions_1h, versions_24h,
    )
}

fn append_line(path: &Path, line: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

pub fn write_stats(kind: &str, state: &HubState, path: &Path) {
    let line = build_log_line(kind, state);
    tracing::info!("Stats: {}", line);
    if let Err(e) = append_line(path, &line) {
        tracing::warn!("Failed to write stats log to {}: {}", path.display(), e);
    }
}
