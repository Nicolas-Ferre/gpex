#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

is_mod_statement() {
    [[ $line =~ mod[[:space:]][a-zA-Z0-9_]+\; ]]
}

is_comment() {
    [[ $line =~ ^[[:space:]]*// ]]
}

is_attribute() {
    [[ $line =~ ^# ]]
}

is_empty_line() {
    [[ $line =~ ^[[:space:]]*$ ]]
}

exit_code=0
while read -r -d '' file; do
    is_not_mod_item_found=false
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if ! is_mod_statement && ! is_comment && ! is_empty_line && ! is_attribute; then
            is_not_mod_item_found=true
        elif is_mod_statement && [[ $is_not_mod_item_found == true ]]; then
            echo "$file:$line_number: inline mod statements should be at the top of the file"
            exit_code=1
            break
        fi
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)
exit $exit_code
