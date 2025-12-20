#!/bin/bash
set -xeu

coverage=$(grep "<coverage" coverage.xml | grep -oP 'line-rate="\K[0-9.]+')
failure=$(awk 'BEGIN{ print '"$coverage * 100"'<'"$COV_THRESHOLD"' }')
if [ "$failure" -eq "1" ]; then
    echo "Coverage has failed with $coverage% instead of at least $COV_THRESHOLD%."
    exit 1
else
    echo "Coverage has successfully passed with $coverage%."
fi
