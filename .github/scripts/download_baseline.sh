#!/bin/bash
set -e

echo "Looking for latest 'test_results' artifact..."
# GITHUB_REPOSITORY and GITHUB_TOKEN are expected to be set in the environment
ARTIFACT_ID=$(curl -s -H "Authorization: Bearer $GITHUB_TOKEN" \
    "https://api.github.com/repos/$GITHUB_REPOSITORY/actions/artifacts" \
| jq -r '.artifacts[] | select(.name=="test_results") | .id' | sort -n | tail -n1)

if [ -z "$ARTIFACT_ID" ] || [ "$ARTIFACT_ID" = "null" ]; then
    echo "No baseline artifact found; creating default baseline file"
    mkdir -p .baseline
    echo '{"cargo_test":{"passed":0,"failed":0}}' > .baseline/test_results.json
else
    echo "Found artifact id: $ARTIFACT_ID; downloading..."
    mkdir -p tmp_art && curl -sL -H "Authorization: Bearer $GITHUB_TOKEN" -H "Accept: application/vnd.github.v3+json" \
    "https://api.github.com/repos/$GITHUB_REPOSITORY/actions/artifacts/$ARTIFACT_ID/zip" -o tmp_art/baseline.zip
    unzip -o tmp_art/baseline.zip -d tmp_art
    # find the test_results.json inside the unzipped artifact and move it to .baseline
    TARGET=$(find tmp_art -type f -name test_results.json | head -n1)
    if [ -z "$TARGET" ]; then
        echo "Artifact did not contain test_results.json; creating default baseline"
        mkdir -p .baseline
        echo '{"cargo_test":{"passed":0,"failed":0}}' > .baseline/test_results.json
    else
        mkdir -p .baseline
        mv "$TARGET" .baseline/test_results.json
    fi
    rm -rf tmp_art
fi
