#!/bin/bash
# analyze_regression.sh - Compare current test results against committed baseline
# Usage: ./analyze_regression.sh <current_results.json> <baseline.json>
# Output: JSON with regressions, fixes, and summary

set -e

CURRENT="$1"
BASELINE="$2"

if [[ ! -f "$CURRENT" ]]; then
    echo '{"error": "Current results file not found"}' >&2
    exit 1
fi

if [[ ! -f "$BASELINE" ]]; then
    echo '{"error": "Baseline file not found", "has_baseline": false, "regressions": [], "fixes": []}'
    exit 0
fi

# Check if baseline is initial (never updated)
BASELINE_STATUS=$(jq -r '.status' "$BASELINE")
if [[ "$BASELINE_STATUS" == "INITIAL" ]]; then
    echo '{"has_baseline": false, "regressions": [], "fixes": [], "message": "No baseline established yet"}'
    exit 0
fi

# Extract failure IDs from both files
BASELINE_FAILURES=$(jq -r '[.failures[].id] | sort | .[]' "$BASELINE" 2>/dev/null || echo "")
CURRENT_FAILURES=$(jq -r '[.failures[].id] | sort | .[]' "$CURRENT" 2>/dev/null || echo "")

# Handle empty lists
if [[ -z "$BASELINE_FAILURES" ]]; then
    BASELINE_FAILURES=""
fi
if [[ -z "$CURRENT_FAILURES" ]]; then
    CURRENT_FAILURES=""
fi

# Find regressions: in current but not in baseline (newly failing)
if [[ -n "$CURRENT_FAILURES" && -n "$BASELINE_FAILURES" ]]; then
    REGRESSIONS=$(comm -23 <(echo "$CURRENT_FAILURES" | sort) <(echo "$BASELINE_FAILURES" | sort) | jq -R -s 'split("\n") | map(select(length > 0))')
    elif [[ -n "$CURRENT_FAILURES" ]]; then
    REGRESSIONS=$(echo "$CURRENT_FAILURES" | jq -R -s 'split("\n") | map(select(length > 0))')
else
    REGRESSIONS="[]"
fi

# Find fixes: in baseline but not in current (now passing)
if [[ -n "$BASELINE_FAILURES" && -n "$CURRENT_FAILURES" ]]; then
    FIXES=$(comm -13 <(echo "$CURRENT_FAILURES" | sort) <(echo "$BASELINE_FAILURES" | sort) | jq -R -s 'split("\n") | map(select(length > 0))')
    elif [[ -n "$BASELINE_FAILURES" ]]; then
    FIXES=$(echo "$BASELINE_FAILURES" | jq -R -s 'split("\n") | map(select(length > 0))')
else
    FIXES="[]"
fi

# Get detailed info for regressions
REGRESSION_DETAILS=$(jq --argjson ids "$REGRESSIONS" '[.failures[] | select(.id as $id | $ids | index($id))]' "$CURRENT")

# Get detailed info for fixes (from baseline)
FIX_DETAILS=$(jq --argjson ids "$FIXES" '[.failures[] | select(.id as $id | $ids | index($id))]' "$BASELINE")

# Build output JSON
jq -n \
--argjson regressions "$REGRESSION_DETAILS" \
--argjson fixes "$FIX_DETAILS" \
--argjson baseline_passed "$(jq '.cargo_test.passed' "$BASELINE")" \
--argjson baseline_failed "$(jq '.cargo_test.failed' "$BASELINE")" \
--argjson current_passed "$(jq '.cargo_test.passed' "$CURRENT")" \
--argjson current_failed "$(jq '.cargo_test.failed' "$CURRENT")" \
--arg baseline_timestamp "$(jq -r '.timestamp_human' "$BASELINE")" \
'{
        has_baseline: true,
        regressions: $regressions,
        fixes: $fixes,
        regression_count: ($regressions | length),
        fix_count: ($fixes | length),
        baseline: {
            timestamp: $baseline_timestamp,
            passed: $baseline_passed,
            failed: $baseline_failed
        },
        current: {
            passed: $current_passed,
            failed: $current_failed
        },
        delta_passed: ($current_passed - $baseline_passed),
        delta_failed: ($current_failed - $baseline_failed)
}'
