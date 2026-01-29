#!/bin/bash
set -e

# Install DeepSource CLI
curl -s https://deepsource.com/cli | sh

# If DEEPSOURCE_DSN is provided (secret), use it; otherwise fall back to OIDC
if [ -n "$DEEPSOURCE_DSN" ]; then
  echo "Using DEEPSOURCE_DSN from secrets to report coverage"
  ./bin/deepsource report --analyzer test-coverage --key rust --value-file lcov.info
else
  echo "DEEPSOURCE_DSN not set; attempting to use OIDC to report coverage"
  ./bin/deepsource report --analyzer test-coverage --key rust --value-file lcov.info --use-oidc
fi

