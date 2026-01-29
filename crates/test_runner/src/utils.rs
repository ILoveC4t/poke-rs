use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn find_project_root() -> PathBuf {
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

pub fn save_test_run(timestamp: &str, json: &str) {
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

pub fn get_timestamps() -> (String, String) {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();

    // Unix timestamp for filename
    let timestamp = format!("{}", secs);

    // Human readable (basic ISO-ish format without chrono)
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
