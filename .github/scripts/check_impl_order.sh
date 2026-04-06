#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

# Keep only string of form "<type>" or "<trait> for <type>", without generics.
strip_impl_line() {
    sed \
        -e ":a" \
        -e "s/<[^<>]*>//g" \
        -e "ta" \
        -e "s/}$//g" \
        -e "s/ {$//g"
}

exit_code=0
while read -r -d '' file; do
    not_trait_impl_block_types=()
    while IFS= read -r line; do
        type=$(echo "$line" | grep -oe "\w*$")
        is_trait_impl=$(echo "$line" | grep -o " for " || true)
        if in_array "$type" "${not_trait_impl_block_types[@]-}"; then
            echo "$file: '$type' type has incorrect impl block order (trait impl blocks should be before type impl block)."
            exit_code=1
        fi
        if [[ -z $is_trait_impl ]]; then
            not_trait_impl_block_types+=("$type")
        fi
    done < <(grep -e "^impl" "$file" | strip_impl_line)
done < <(find src/ tests/ -type f -name "*.rs" -print0)
exit $exit_code
