#!/bin/bash
set -e

# Generate coverage using cargo-llvm-cov
# (Cleans first to avoid stale artifacts)
cargo llvm-cov clean --workspace
cargo llvm-cov run -p test_runner -- run
cargo llvm-cov report --lcov --output-path lcov.info

