#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

VISIBILITY_REGEX='(pub(\([^)]*\))?[[:space:]]+)?'
FUNCTION_MODIFIER_REGEX='((async|unsafe|const|default)[[:space:]]+|extern([[:space:]]+"[^"]*")?[[:space:]]+)*'
MOD_REGEX="^${VISIBILITY_REGEX}mod[[:space:]]+(r#)?[a-zA-Z_][a-zA-Z0-9_]*;"
USE_REGEX="^${VISIBILITY_REGEX}use[[:space:]]+"
CONST_REGEX="^${VISIBILITY_REGEX}const[[:space:]]+[a-zA-Z_]"
STATIC_REGEX="^${VISIBILITY_REGEX}static[[:space:]]+(mut[[:space:]]+)?[a-zA-Z_]"
TYPE_REGEX="^${VISIBILITY_REGEX}(unsafe[[:space:]]+)?(type|struct|enum|union|trait)([[:space:]]|<)"
IMPL_REGEX="^(unsafe[[:space:]]+)?impl([[:space:]]|<)"
FUNCTION_REGEX="^${VISIBILITY_REGEX}${FUNCTION_MODIFIER_REGEX}fn([[:space:]]|<)"
ITEM_KINDS=("module" "use" "const" "static" "type/impl" "free function")

compute_item_rank() {
    item_rank=""
    has_visibility_order=true
    if [[ $line =~ $MOD_REGEX ]]; then
        item_rank=0
    elif [[ $line =~ $USE_REGEX ]]; then
        item_rank=1
    elif [[ $line =~ $FUNCTION_REGEX ]]; then
        item_rank=5 # avoids misclassifying `const fn` items
    elif [[ $line =~ $CONST_REGEX ]]; then
        item_rank=2
    elif [[ $line =~ $STATIC_REGEX ]]; then
        item_rank=3
    elif [[ $line =~ $TYPE_REGEX || $line =~ $IMPL_REGEX ]]; then
        item_rank=4
        if [[ $line =~ $IMPL_REGEX ]]; then
            has_visibility_order=false
        fi
    fi
}

exit_code=0
while read -r -d '' file; do
    highest_rank=-1
    highest_visibility_ranks=(-1 -1 -1 -1 -1 -1)
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        compute_item_rank
        if [[ -z $item_rank ]]; then
            continue
        fi
        if ((item_rank < highest_rank)); then
            echo "$file:$line_number: ${ITEM_KINDS[item_rank]} items should be defined before ${ITEM_KINDS[highest_rank]} items"
            exit_code=1
        elif ((item_rank > highest_rank)); then
            highest_rank=$item_rank
        fi
        if [[ $has_visibility_order == false ]]; then
            continue
        fi
        visibility_rank=$(compute_visibility_rank "$line")
        highest_visibility_rank=${highest_visibility_ranks[item_rank]}
        if ((visibility_rank < highest_visibility_rank)); then
            echo "$file:$line_number: ${VISIBILITY_KINDS[visibility_rank]} ${ITEM_KINDS[item_rank]} items should be defined before ${VISIBILITY_KINDS[highest_visibility_rank]} ${ITEM_KINDS[item_rank]} items"
            exit_code=1
        elif ((visibility_rank > highest_visibility_rank)); then
            highest_visibility_ranks[item_rank]=$visibility_rank
        fi
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit "$exit_code"
