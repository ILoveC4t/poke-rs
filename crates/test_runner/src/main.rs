//! AI-friendly test summary runner.
//!
//! Runs `cargo test` and produces a structured JSON summary suitable for AI agents.
//! Results are saved to `.test_runs/` with versioning.
//!
//! Usage:
//!   cargo run -p test_runner
//!   cargo run -p test_runner -- --package poke_engine
//!   cargo run -p test_runner -- --filter damage

use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

#[derive(Serialize, Default)]
struct TestSummary {
    timestamp: String,
    timestamp_human: String,
    duration_seconds: f64,
    status: String,
    cargo_test: CargoTestResult,
    fixture_results: FixtureResults,
    failures: Vec<FailureInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    warnings: Vec<String>,
}

#[derive(Serialize, Default)]
struct CargoTestResult {
    passed: u32,
    failed: u32,
    ignored: u32,
    total: u32,
}

#[derive(Serialize, Default)]
struct FixtureResults {
    passed: u32,
    failed: u32,
    skipped: u32,
}

#[derive(Serialize)]
struct FailureInfo {
    id: String,
    name: String,
    error: String,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    
    // Build cargo test command
    // We explicitly exclude the runner itself to be safe, though being in a separate crate helps.
    // However, since we are usually running from workspace root, "cargo test" tests everything in workspace.
    // We should probably default to testing only poke_engine if no args provided, or just run everything.
    // Running "cargo test" from root will test test_runner too if it's in workspace.
    // But test_runner has no tests.
    
    let mut cmd_args = vec!["test".to_string(), "--no-fail-fast".to_string()];
    
    // Add package filter if specified
    let mut i = 0;
    let mut test_filter = None;
    let mut package_specified = false;

    while i < args.len() {
        match args[i].as_str() {
            "--package" | "-p" => {
                if i + 1 < args.len() {
                    cmd_args.push("-p".to_string());
                    cmd_args.push(args[i + 1].clone());
                    package_specified = true;
                    i += 2;
                    continue;
                }
            }
            "--filter" => {
                if i + 1 < args.len() {
                    test_filter = Some(args[i + 1].clone());
                    i += 2;
                    continue;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    // Default to testing poke_engine if no package specified?
    // Actually, widespread practice is 'cargo test' tests workspace. 
    // But to solve the locking issue, executing 'cargo test' from within a running binary is tricky if that binary is part of the set being tested/built.
    // Since 'test_runner' is now a separate crate, 'cargo test' (workspace) WILL try to build/test 'test_runner'.
    // So we MUST exclude test_runner from the cargo test command we spawn.
    
    if !package_specified {
        // Exclude ourselves!
        cmd_args.push("--workspace".to_string());
        cmd_args.push("--exclude".to_string());
        cmd_args.push("test_runner".to_string());
    }
    
    // Add filter if present
    if let Some(filter) = test_filter {
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
    // Ensure .test_runs is in the project root
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
    
    // Limit failures to first 20
    summary.failures.truncate(20);
    
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
    // If output status is not success, it might be a build failure or test failure
    // If cargo test returned non-zero, it usually means tests failed OR build failed.
    // If we parsed 0 failures from test output but exit code is error, it's likely a build failure.
    
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
    
    // Output to stdout
    println!("###AI_TEST_SUMMARY_START###");
    println!("{}", json);
    println!("###AI_TEST_SUMMARY_END###");
    
    // Print location info
    eprintln!("\n[test_runner] Results saved to .test_runs/latest.json");
    
    if summary.status == "FAILURE" {
        std::process::exit(1);
    }
}

fn find_project_root() -> PathBuf {
    let mut dir = std::env::current_dir().unwrap();
    loop {
        if dir.join("Cargo.toml").exists() {
            return dir;
        }
        if !dir.pop() {
            return std::env::current_dir().unwrap();
        }
    }
}

fn save_test_run(timestamp: &str, json: &str) {
    let dir = find_project_root();
    let test_runs_dir = dir.join(".test_runs");
    
    // Create directory if it doesn't exist
    if !test_runs_dir.exists() {
        fs::create_dir_all(&test_runs_dir).expect("Failed to create .test_runs directory");
    }
    
    // Write timestamped file
    let timestamped_path = test_runs_dir.join(format!("{}.json", timestamp));
    fs::write(&timestamped_path, json).expect("Failed to write timestamped test run");
    
    // Write/update latest.json
    let latest_path = test_runs_dir.join("latest.json");
    fs::write(&latest_path, json).expect("Failed to write latest.json");
    
    // Cleanup old files (keep last 50)
    cleanup_old_runs(&test_runs_dir, 50);
}

fn cleanup_old_runs(dir: &PathBuf, keep: usize) {
    let mut files: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name().to_string_lossy() != "latest.json"
                && e.path().extension().map(|x| x == "json").unwrap_or(false)
        })
        .collect();
    
    if files.len() <= keep {
        return;
    }
    
    // Sort by name (timestamps sort chronologically)
    files.sort_by_key(|e| e.file_name());
    
    // Remove oldest files
    let to_remove = files.len() - keep;
    for entry in files.into_iter().take(to_remove) {
        let _ = fs::remove_file(entry.path());
    }
}

fn get_timestamps() -> (String, String) {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    let secs = duration.as_secs();
    
    // Unix timestamp for filename
    let timestamp = format!("{}", secs);
    
    // Human readable (basic ISO-ish format without chrono)
    // Calculate approximate datetime
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    
    // Approximate year/month/day calculation
    let mut year = 1970;
    let mut remaining_days = days_since_epoch as i64;
    
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }
    
    let days_in_months: [i64; 12] = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    
    let mut month = 1;
    for days in days_in_months.iter() {
        if remaining_days < *days {
            break;
        }
        remaining_days -= *days;
        month += 1;
    }
    let day = remaining_days + 1;
    
    let human = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    );
    
    (timestamp, human)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn parse_test_result(line: &str) -> Option<(u32, u32, u32)> {
    let mut passed = 0u32;
    let mut failed = 0u32;
    let mut ignored = 0u32;
    
    // Line format examples:
    // "test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out"
    // "test result: FAILED. 10 passed; 1 failed; 0 ignored; ..."
    
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
            // Try to parse the word before 'keyword'
            // Handle "ok." or "FAILED." prefix by just taking the last token
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
