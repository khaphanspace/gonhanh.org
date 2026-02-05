#!/usr/bin/env bash

# This script will build, run and generate coverage reports for the whole
# testsuite using cargo-llvm-cov.
#
# Prerequisites:
#   cargo install cargo-llvm-cov
#   OR (on CI): uses: taiki-e/install-action@cargo-llvm-cov

# Exit the script if an unexpected error arise
set -e
# Treat unset variables as errors
set -u
# Ensure pipeline failures are caught (not just the last command's exit code)
set -o pipefail
# Print expanded commands to stdout before running them
set -x

ME="${0}"
ME_dir="$(dirname -- "$(readlink -f -- "${ME}")")"
REPO_main_dir="$(dirname -- "${ME_dir}")"

COVERAGE_DIR=${COVERAGE_DIR:-"${REPO_main_dir}/coverage"}
REPORT_PATH="${COVERAGE_DIR}/report/total.lcov.info"

mkdir -p "${COVERAGE_DIR}/report"

cd "${REPO_main_dir}/core"

# Clean previous coverage data
cargo llvm-cov clean --workspace

# Run tests with coverage and generate lcov report
cargo llvm-cov nextest \
    --all-features \
    --lcov \
    --output-path "${REPORT_PATH}" \
    --ignore-filename-regex '(tests/|build\.rs)'

# Notify the report file to github
if [ -n "${GITHUB_OUTPUT:-}" ]; then
    echo "report=${REPORT_PATH}" >> "${GITHUB_OUTPUT}"
fi

echo "## Coverage report generated at ${REPORT_PATH}"
if [ -f "${REPORT_PATH}" ]; then
    # Show summary using cargo llvm-cov report
    cargo llvm-cov report --summary-only
fi

echo "Done!"
