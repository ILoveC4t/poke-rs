use clap::Args;
use std::process::Command;
use std::time::Instant;
use std::fs;
use crate::models::{TestSummary, FailureInfo};
use crate::utils::{find_project_root, get_timestamps, save_test_run};

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Specific package to test
    #[arg(short, long)]
    pub package: Option<String>,

    /// Filter test cases by name substring
    #[arg(short, long)]
    pub filter: Option<String>,
}

pub fn execute(args: RunArgs) {
    let mut cmd_args = vec!["test".to_string(), "--no-fail-fast".to_string()];
    
    if let Some(pkg) = args.package {
        cmd_args.push("-p".to_string());
        cmd_args.push(pkg);
    } else {
        // Exclude ourselves if running workspace tests
        cmd_args.push("--workspace".to_string());
        cmd_args.push("--exclude".to_string());
        cmd_args.push("test_runner".to_string());
    }
    
    if let Some(filter) = args.filter {
        cmd_args.push(filter);
    }
    
    // Add test args after --
    cmd_args.push("--".to_string());
    cmd_args.push("--nocapture".to_string());
    
    let start = Instant::now();
    
    // Run cargo test and capture all output
    let output = Command::new("cargo")
        .args(&cmd_args)
        .output()
        .expect("Failed to run cargo test");
    
    let duration = start.elapsed();
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    
    let mut summary = TestSummary::default();
    
    // Combine stdout and stderr for parsing (cargo outputs to both)
    let all_output = format!("{}\n{}", stdout, stderr);
    let stdout_lines: Vec<&str> = all_output.lines().collect();

    // Debug: Save raw output to file for inspection
    let root_dir = find_project_root();
    let test_runs_dir = root_dir.join(".test_runs");
    if !test_runs_dir.exists() {
        let _ = fs::create_dir_all(&test_runs_dir);
    }
    if test_runs_dir.exists() {
         let _ = fs::write(test_runs_dir.join("last_output.txt"), &all_output);
    }
    
    // Extract warnings
    let warnings: Vec<String> = stderr
        .lines()
        .filter(|l| l.contains("warning:") && !l.contains("unused"))
        .take(10)
        .map(|s| s.to_string())
        .collect();
    
    // Parse output
    for line in &stdout_lines {
        // Parse cargo test summary: "test result: ok. X passed; Y failed; Z ignored"
        if line.starts_with("test result:") {
            if let Some(caps) = parse_test_result(line) {
                // Accumulate results from multiple test binaries
                summary.cargo_test.passed += caps.0;
                summary.cargo_test.failed += caps.1;
                summary.cargo_test.ignored += caps.2;
            }
        }
        
        // Parse custom fixture output: "Passed:  214"
        if line.starts_with("Passed:") {
            if let Some(n) = parse_number_after_colon(line) {
                summary.fixture_results.passed = n;
            }
        }
        if line.starts_with("Failed:") {
            if let Some(n) = parse_number_after_colon(line) {
                summary.fixture_results.failed = n;
            }
        }
        if line.starts_with("Skipped:") {
            if let Some(n) = parse_number_after_colon(line) {
                summary.fixture_results.skipped = n;
            }
        }
        
        // Parse failure entries: "  [id] name: error"
        if line.trim_start().starts_with('[') {
            if let Some(failure) = parse_failure_line(line) {
                summary.failures.push(failure);
            }
        }
    }
    
    // Calculate total
    summary.cargo_test.total = summary.cargo_test.passed 
        + summary.cargo_test.failed 
        + summary.cargo_test.ignored;
    
    // Deduplicate and limit warnings
    let mut unique_warnings: Vec<String> = warnings
        .into_iter()
        .filter(|w| !w.contains("unused"))
        .take(10)
        .collect();
    unique_warnings.dedup();
    summary.warnings = unique_warnings;
    
    // Set metadata
    let (timestamp, timestamp_human) = get_timestamps();
    summary.timestamp = timestamp.clone();
    summary.timestamp_human = timestamp_human;
    summary.duration_seconds = duration.as_secs_f64();
    
    // Determine status (SUCCESS/FAILURE)
    let exit_success = output.status.success();
    let has_failures = summary.cargo_test.failed > 0 || summary.fixture_results.failed > 0;
    
    summary.status = if exit_success && !has_failures {
        "SUCCESS".to_string()
    } else {
        "FAILURE".to_string()
    };
    
    // If no tests passed/failed and we have an error status, ensure we log that it might be a build error
    if !exit_success && summary.cargo_test.total == 0 && summary.failures.is_empty() {
         summary.failures.push(FailureInfo {
             id: "BUILD_ERROR".to_string(),
             name: "Cargo Build".to_string(),
             error: "Process exited with error code (Check last_output.txt)".to_string(),
         });
    }

    // Write to .test_runs directory
    let json = serde_json::to_string_pretty(&summary).unwrap();
    save_test_run(&timestamp, &json);
    
    let root_dir = find_project_root();
    let latest_path = root_dir.join(".test_runs/latest.json");
    
    // Output concise summary to console
    println!("\n=== Test Run Summary ===");
    println!("Status:      {}", summary.status);
    println!("Time:        {:.2}s", summary.duration_seconds);
    println!("Cargo Tests: {} passed, {} failed, {} ignored", 
             summary.cargo_test.passed, summary.cargo_test.failed, summary.cargo_test.ignored);
    if summary.fixture_results.passed + summary.fixture_results.failed > 0 {
        println!("Fixtures:    {} passed, {} failed, {} skipped",
                 summary.fixture_results.passed, summary.fixture_results.failed, summary.fixture_results.skipped);
    }
    
    if !summary.failures.is_empty() {
        println!("\n{} failures detected. See full report in:", summary.failures.len());
    } else {
        println!("\nFull report saved to:");
    }
    println!("{}", latest_path.display());
    println!("========================");
    
    eprintln!("\n[test_runner] Results saved to .test_runs/latest.json");
}

// Helpers specific to parsing (could be moved to parsing.rs if needed, but keeping here for now)
fn parse_test_result(line: &str) -> Option<(u32, u32, u32)> {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut ignored = 0u32;
    
    for part in line.split(';') {
        let part = part.trim();
        if part.contains("passed") {
            if let Some(n) = extract_number_before(part, "passed") {
                passed = n;
            }
        } else if part.contains("failed") {
            if let Some(n) = extract_number_before(part, "failed") {
                failed = n;
            }
        } else if part.contains("ignored") {
            if let Some(n) = extract_number_before(part, "ignored") {
                ignored = n;
            }
        }
    }
    
    Some((passed, failed, ignored))
}

fn extract_number_before(text: &str, keyword: &str) -> Option<u32> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    for i in 0..parts.len() {
        if parts[i] == keyword && i > 0 {
            if let Ok(n) = parts[i-1].parse() {
                return Some(n);
            }
        }
    }
    None
}

fn parse_number_after_colon(line: &str) -> Option<u32> {
    let after_colon = line.split(':').nth(1)?;
    after_colon.trim().split_whitespace().next()?.parse().ok()
}

fn parse_failure_line(line: &str) -> Option<FailureInfo> {
    let trimmed = line.trim();
    if !trimmed.starts_with('[') {
        return None;
    }
    
    let end_bracket = trimmed.find(']')?;
    let id = trimmed[1..end_bracket].to_string();
    
    let rest = trimmed[end_bracket + 1..].trim();
    let colon_pos = rest.find(':')?;
    let name = rest[..colon_pos].trim().to_string();
    let error = rest[colon_pos + 1..].trim().to_string();
    
    Some(FailureInfo { id, name, error })
}
