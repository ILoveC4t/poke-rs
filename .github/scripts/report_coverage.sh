#!/bin/bash
set -e

# Report coverage to DeepSource
curl https://deepsource.com/cli | sh
./bin/deepsource report --analyzer test-coverage --key rust --value-file lcov.info
