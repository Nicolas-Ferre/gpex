#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/config.sh"

show_error() {
    local message="$1"
    echo "$file: $message"
    exit_code=1
}

exit_code=0

while read -r -d '' file; do
    split_path=' '$(echo "$file" |
        sed -E 's/([a-z])([A-Z])/\1 \2/g' |
        tr '_/-' ' ' |
        tr '[:upper:]' '[:lower:]')' '
    for word in "${FORBIDDEN_WORDS[@]}"; do
        if [[ $split_path =~ [[:space:]]"$word"[[:space:]] ]]; then
            show_error "file path contains forbidden word '$word'"
        fi
        if [[ $split_path =~ [[:space:]]"$word"s[[:space:]] ]]; then
            show_error "file path contains forbidden word '${word}s'"
        fi
    done
done < <(find src/ tests/ -type f -print0)

exit "$exit_code"
