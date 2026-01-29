use crate::models::{FailureInfo, TestSummary};
use crate::utils::find_project_root;
use clap::Args;
use std::fs;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct AnalyzeArgs {
    /// Base run to compare against (default: previous run)
    #[arg(short, long)]
    pub base: Option<String>,

    /// Head run to check (default: latest)
    #[arg(long, default_value = "latest")]
    pub head: String,
}

pub fn execute(args: AnalyzeArgs) {
    let root = find_project_root();
    let runs_dir = root.join(".test_runs");

    // Load Head
    let head_summary = load_run(&runs_dir, &args.head).expect("Failed to load head run");

    // Determine Base
    let base_name = if let Some(base) = args.base {
        if base == "oldest" {
            find_oldest(&runs_dir).expect("No runs found to compare against")
        } else {
            base
        }
    } else {
        // Find predecessor of head
        find_predecessor(&runs_dir, &head_summary.timestamp)
            .expect("Could not find a previous run to compare against")
    };

    let base_summary = load_run(&runs_dir, &base_name).expect("Failed to load base run");

    println!("\n=== Test Regression Analysis ===");
    println!("Comparing:");
    println!(
        "  Base: {} ({})",
        base_summary.timestamp, base_summary.timestamp_human
    );
    println!(
        "  Head: {} ({})",
        head_summary.timestamp, head_summary.timestamp_human
    );
    println!("================================");

    // Analyze Regressions (Passed in Base -> Failed in Head)
    let base_failures: Vec<String> = base_summary.failures.iter().map(|f| f.id.clone()).collect();
    let head_failures: Vec<String> = head_summary.failures.iter().map(|f| f.id.clone()).collect();

    let regressions: Vec<&FailureInfo> = head_summary
        .failures
        .iter()
        .filter(|f| !base_failures.contains(&f.id))
        .collect();

    let fixed: Vec<&FailureInfo> = base_summary
        .failures
        .iter()
        .filter(|f| !head_failures.contains(&f.id))
        .collect();

    if regressions.is_empty() && fixed.is_empty() {
        println!("No regressions or fixes detected.");
        return;
    }

    if !regressions.is_empty() {
        println!("\n🔴 REGRESSIONS ({}):", regressions.len());
        for f in regressions {
            println!("  - {} ({})", f.name, f.id);
            println!("    Error: {}", f.error);
        }
    }

    if !fixed.is_empty() {
        println!("\n🟢 FIXED ({}):", fixed.len());
        for f in fixed {
            println!("  - {} ({})", f.name, f.id);
        }
    }

    println!("\n================================");
}

fn load_run(dir: &PathBuf, name: &str) -> Option<TestSummary> {
    let filename = if name.ends_with(".json") {
        name.to_string()
    } else {
        format!("{}.json", name)
    };
    let path = dir.join(filename);

    if !path.exists() {
        eprintln!("Run file not found: {}", path.display());
        return None;
    }

    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn get_sorted_runs(dir: &PathBuf) -> Option<Vec<String>> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name != "latest.json" && name != "last_output.txt" && name.ends_with(".json")
        })
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();

    // Sort chronologically
    files.sort();
    Some(files)
}

fn find_oldest(dir: &PathBuf) -> Option<String> {
    let files = get_sorted_runs(dir)?;
    files.first().cloned()
}

fn find_predecessor(dir: &PathBuf, current_timestamp: &str) -> Option<String> {
    let files = get_sorted_runs(dir)?;

    // Find position of current
    // Note: current_timestamp matches the filename prefix
    let current_filename = format!("{}.json", current_timestamp);

    if let Ok(idx) = files.binary_search(&current_filename) {
        if idx > 0 {
            return Some(files[idx - 1].clone());
        }
    } else {
        // If exact match not found (maybe measuring "latest" which is a copy),
        // try to find the one just before where it would be.
        // But "latest.json" content has a timestamp.
        // If we loaded "latest", we have the timestamp.
        // We are searching for that timestamp in the list of files.
        // It SHOULD be there if we saved it correctly.

        // If not found, look for last file before this timestamp
        let current_ts_val: u64 = current_timestamp.parse().unwrap_or(0);

        for file in files.iter().rev() {
            let file_ts_str = file.trim_end_matches(".json");
            let file_ts: u64 = file_ts_str.parse().unwrap_or(0);
            if file_ts < current_ts_val {
                return Some(file.clone());
            }
        }
    }

    None
}
