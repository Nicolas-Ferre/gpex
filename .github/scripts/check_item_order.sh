#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

VISIBILITY_REGEX='(pub(\([^)]*\))?[[:space:]]+)?'
FUNCTION_MODIFIER_REGEX='((async|unsafe|const|default)[[:space:]]+|extern([[:space:]]+"[^"]*")?[[:space:]]+)*'
MOD_REGEX="^${VISIBILITY_REGEX}mod[[:space:]]+"
USE_REGEX="^${VISIBILITY_REGEX}use[[:space:]]+"
CONST_REGEX="^${VISIBILITY_REGEX}const[[:space:]]+[a-zA-Z_]"
STATIC_REGEX="^${VISIBILITY_REGEX}static[[:space:]]+(mut[[:space:]]+)?[a-zA-Z_]"
TYPE_REGEX="^${VISIBILITY_REGEX}(unsafe[[:space:]]+)?(type|struct|enum|union|trait)([[:space:]]|<)"
IMPL_REGEX="^(unsafe[[:space:]]+)?impl([[:space:]]|<)"
FUNCTION_REGEX="^${VISIBILITY_REGEX}${FUNCTION_MODIFIER_REGEX}fn([[:space:]]|<)"
ITEM_KINDS=("module" "use" "const" "static" "type" "impl" "free function")

get_item_rank() {
    if [[ $line =~ $MOD_REGEX ]]; then
        echo 0
    elif [[ $line =~ $USE_REGEX ]]; then
        echo 1
    elif [[ $line =~ $FUNCTION_REGEX ]]; then
        echo 6
    elif [[ $line =~ $CONST_REGEX ]]; then
        echo 2
    elif [[ $line =~ $STATIC_REGEX ]]; then
        echo 3
    elif [[ $line =~ $TYPE_REGEX ]]; then
        echo 4
    elif [[ $line =~ $IMPL_REGEX ]]; then
        echo 5
    fi
}

exit_code=0
while read -r -d '' file; do
    highest_rank=-1
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        item_rank=$(get_item_rank)
        if [[ -z $item_rank ]]; then
            continue
        fi
        if ((item_rank < highest_rank)); then
            echo "$file:$line_number: ${ITEM_KINDS[item_rank]} items should be defined before ${ITEM_KINDS[highest_rank]} items"
            exit_code=1
        elif ((item_rank > highest_rank)); then
            highest_rank=$item_rank
        fi
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit "$exit_code"
