#!/bin/bash
set -euo pipefail

find ./ \
    -type f \( -name "*.rs" -o -name "*.gpex" -o -name "*.toml" -o -name "*.yaml" -o -name "*.yml" \) \
    ! -path "./target/*" \
    -exec grep -U -l $'\r' {} + |
    tee /dev/stderr | grep -q . &&
    exit 1 || exit 0
