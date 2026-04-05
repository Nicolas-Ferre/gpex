#!/bin/bash
set -euo pipefail

COVERAGE_THRESHOLD=1.0 # Require 100% line coverage (except lines excluded explicitly)

command -v xmllint &>/dev/null || {
    echo "xmllint is not installed."
    exit 1
}
[[ -f coverage.xml ]] || {
    echo "coverage.xml not found."
    exit 1
}

coverage=$(xmllint --xpath 'string(//coverage/@line-rate)' coverage.xml)
[[ -n "$coverage" ]] || {
    echo "Could not extract coverage from coverage.xml."
    exit 1
}
[[ "$coverage" =~ ^[0-1]\.[0-9]+$ ]] || {
    echo "Invalid coverage value: '$coverage'."
    exit 1
}
awk "BEGIN { if ($coverage < $COVERAGE_THRESHOLD) exit 1 }" || {
    echo "Coverage check failed: $coverage < $COVERAGE_THRESHOLD"
    exit 1
}
echo "Coverage has successfully passed."
