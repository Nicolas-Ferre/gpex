#!/bin/bash
set -euo pipefail

# shellcheck disable=SC1091
source "$(dirname "$0")/config.sh"

exit_code=0
while read -r -d '' file; do
    line_count=$(wc -l <"$file")
    if ((line_count > MAX_RUST_FILE_LINE_COUNT)); then
        echo "$file: file has too many lines ($line_count > $MAX_RUST_FILE_LINE_COUNT)"
        exit_code=1
    fi
done < <(find src/ tests/ -type f -name "*.rs" -print0)
exit "$exit_code"
