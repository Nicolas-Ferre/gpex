#!/bin/bash
set -euo pipefail

strip_impl_line() {
    sed \
        -e ":a" \
        -e "s/<[^<>]*>//g" \
        -e "ta" -e "s/&//g" \
        -e "s/^impl //g" \
        -e "s/}$//g" \
        -e "s/ {$//g"
}

exit_code=0
while read -r file; do
    not_trait_impl_block_types=""
    while read -r line; do
        type=$(echo "$line" | grep -oe "[a-zA-Z0-9_]*$")
        is_trait_impl=$(echo "$line" | grep -o " for " || echo "")
        if [[ $not_trait_impl_block_types == *" $type "* ]]; then
            echo "$file: '$type' type has incorrect impl block order (trait impl blocks should be before type impl block)."
            exit_code=1
        fi
        if [[ -z $is_trait_impl ]]; then
            not_trait_impl_block_types="$not_trait_impl_block_types $type "
        fi
    done < <((grep -e "^impl" "$file" || true) | strip_impl_line)
done < <(find src/ tests/ -name "*.rs")

if [[ $exit_code -eq 0 ]]; then
    echo "All impl blocks are in correct order."
fi

exit $exit_code
