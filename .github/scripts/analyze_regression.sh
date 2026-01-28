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

# Extract current test summary
PASSED=$(jq -r '.cargo_test.passed' "$CURRENT")
FAILED=$(jq -r '.cargo_test.failed' "$CURRENT")
IGNORED=$(jq -r '.cargo_test.ignored' "$CURRENT")
DURATION=$(jq -r '.duration_seconds' "$CURRENT")
FIX_PASSED=$(jq -r '.fixture_results.passed' "$CURRENT")
FIX_FAILED=$(jq -r '.fixture_results.failed' "$CURRENT")
FIX_SKIPPED=$(jq -r '.fixture_results.skipped' "$CURRENT")

# Extract analysis results
HAS_BASELINE=$(jq -r '.has_baseline // false' "$ANALYSIS")
REGRESSION_COUNT=$(jq -r '.regression_count // 0' "$ANALYSIS")
FIX_COUNT=$(jq -r '.fix_count // 0' "$ANALYSIS")
DELTA_PASSED=$(jq -r '.delta_passed // 0' "$ANALYSIS")
DELTA_FAILED=$(jq -r '.delta_failed // 0' "$ANALYSIS")
BASELINE_TIMESTAMP=$(jq -r '.baseline.timestamp // "N/A"' "$ANALYSIS")

# Determine status
if [[ "$REGRESSION_COUNT" -gt 0 ]]; then
    STATUS_EMOJI="❌"
    STATUS_TEXT="Regressions detected!"
    elif [[ "$FIX_COUNT" -gt 0 ]]; then
    STATUS_EMOJI="🎉"
    STATUS_TEXT="Tests fixed!"
    elif [[ "$FAILED" -eq 0 ]]; then
    STATUS_EMOJI="✅"
    STATUS_TEXT="All tests passing!"
else
    STATUS_EMOJI="⚠️"
    STATUS_TEXT="Some tests failing (no new regressions)"
fi

# Format duration
DURATION_FMT=$(printf "%.2f" "$DURATION")

# Generate markdown
cat << EOF
## ${STATUS_EMOJI} Test Results

**Status:** ${STATUS_TEXT}

### Test Summary

| Metric | Count | Delta |
|--------|-------|-------|
| ✅ Passed | ${PASSED} | ${DELTA_PASSED:+$([ "$DELTA_PASSED" -gt 0 ] && echo "+")${DELTA_PASSED}} |
| ❌ Failed | ${FAILED} | ${DELTA_FAILED:+$([ "$DELTA_FAILED" -gt 0 ] && echo "+")${DELTA_FAILED}} |
| ⏭️ Ignored | ${IGNORED} | - |
| ⏱️ Duration | ${DURATION_FMT}s | - |

### Fixture Results

| Metric | Count |
|--------|-------|
| ✅ Passed | ${FIX_PASSED} |
| ❌ Failed | ${FIX_FAILED} |
| ⏭️ Skipped | ${FIX_SKIPPED} |

EOF

# Show baseline info
if [[ "$HAS_BASELINE" == "true" ]]; then
    echo "<details>"
    echo "<summary>📊 Compared against baseline from ${BASELINE_TIMESTAMP}</summary>"
    echo ""
    
    # Show regressions
    if [[ "$REGRESSION_COUNT" -gt 0 ]]; then
        echo "### 🔴 Regressions (${REGRESSION_COUNT})"
        echo ""
        echo "These tests were passing in the baseline but are now failing:"
        echo ""
        jq -r '.regressions[] | "- **\(.name)** (\(.id))\n  > \(.error)"' "$ANALYSIS"
        echo ""
    fi
    
    # Show fixes
    if [[ "$FIX_COUNT" -gt 0 ]]; then
        echo "### 🟢 Fixed (${FIX_COUNT})"
        echo ""
        echo "These tests were failing in the baseline but are now passing:"
        echo ""
        jq -r '.fixes[] | "- **\(.name)** (\(.id))"' "$ANALYSIS"
        echo ""
    fi
    
    if [[ "$REGRESSION_COUNT" -eq 0 && "$FIX_COUNT" -eq 0 ]]; then
        echo ""
        echo "_No changes from baseline._"
        echo ""
    fi
    
    echo "</details>"
else
    echo ""
    echo "> ℹ️ No baseline established yet. Baseline will be created when this PR merges to main."
fi

echo ""
echo "---"
echo "*Generated by [test workflow](../actions/workflows/test.yml) • Commit: \`${GITHUB_SHA:0:7}\`*"
