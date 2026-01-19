use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct TestSummary {
    pub timestamp: String,
    pub timestamp_human: String,
    pub duration_seconds: f64,
    pub status: String,
    pub cargo_test: CargoTestResult,
    pub fixture_results: FixtureResults,
    pub failures: Vec<FailureInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct CargoTestResult {
    pub passed: u32,
    pub failed: u32,
    pub ignored: u32,
    pub total: u32,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct FixtureResults {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FailureInfo {
    pub id: String,
    pub name: String,
    pub error: String,
}
