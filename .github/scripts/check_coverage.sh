#!/bin/bash
set -xeuo pipefail

coverage=$(grep "<coverage" coverage.xml | grep -oP 'line-rate="\K[0-9.]+' | head -1)
failure=$(awk 'BEGIN{ print '"$coverage"'<'"1.0"' }')
if [ "$failure" -eq "1" ]; then
    echo "Coverage rate has failed with $coverage (expected: 1.0)."
    exit 1
else
    echo "Coverage has successfully passed with $coverage%."
fi
