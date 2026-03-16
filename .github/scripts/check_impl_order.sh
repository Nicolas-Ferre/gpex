#!/bin/bash
set -euo pipefail

find src/ tests/ -name "*.rs" | while read -r file ; do
    not_trait_impl_block_types=""
    (grep "$file" -e "^impl" || true) \
        | sed  \
            -e ":a" \
            -e "s/<[^<>]*>//g" \
            -e "ta" -e "s/&//g" \
            -e "s/^impl //g" \
            -e "s/}$//g" \
            -e "s/ {$//g" \
    | while read -r line ; do
        type=$(echo "$line" | grep -oe "[a-zA-Z0-9_]*$")
        is_trait_impl=$(echo "$line" | grep -o " for " || echo "")
        if [[ $not_trait_impl_block_types == *" $type "* ]]; then
            echo "'$type' type has incorrect impl block order (trait impl blocks should be before type impl block)."
            exit 1
        fi
        if [[ -z $is_trait_impl ]]; then
            not_trait_impl_block_types="$not_trait_impl_block_types $type "
        fi
    done
done

echo "All impl blocks are in correct order."
