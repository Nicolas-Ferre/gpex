#!/bin/bash
set -euo pipefail

previous_line=""

# TODO: first loop on files (previous_line is reset for each file)
# TODO: support not consecutive impl for same type (can be done by storing in list the already found type impl blocks)
grep -rh src/ tests/ -e "^impl" | sed -E ":a;s/<[^<>]*>//g;ta; s/&//g; s/^impl //g; s/ \{}?$//g" | rev | sort | rev | while read -r line ; do
    if [[ -n $previous_line ]]; then
        previous_type=$(echo "$previous_line" | grep -oe "[a-zA-Z0-9_]*$")
        current_type=$(echo "$line" | grep -oe "[a-zA-Z0-9_]*$")
        if [[ $previous_type == $current_type ]]; then
            is_previous_trait_impl=$(echo "$previous_line" | grep -o " for " || echo "")
            is_current_trait_impl=$(echo "$line" | grep -o " for " || echo "")
            if [[ -z $is_previous_trait_impl && -n $is_current_trait_impl ]]; then
                echo "'$current_type' type has incorrect impl block order (trait impl blocks should be before type impl block)."
                exit 1
            fi
        fi
    fi
    previous_line="$line"
done

echo "All impl blocks are in correct order."
