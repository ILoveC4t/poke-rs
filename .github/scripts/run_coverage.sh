#!/bin/bash
set -e

# Run test runner with coverage
cargo llvm-cov clean --workspace
cargo llvm-cov run -p test_runner -- run
cargo llvm-cov report --lcov --output-path lcov.info
