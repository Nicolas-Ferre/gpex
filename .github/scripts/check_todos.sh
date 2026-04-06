#!/bin/bash
set -euo pipefail
IFS=

find ./ -type f \
    \( -name "*.rs" -o -name "*.gpex" -o -name "*.toml" -o -name "*.yaml" -o -name "*.yml" \) \
    ! -path "./target/*" ! -path "./.github/*" \
    -exec grep -U -l $'TODO' {} + |
    tee /dev/stderr | grep -q . &&
    exit 1 || exit 0
