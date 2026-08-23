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
IMPL_END_REGEX="^}"
EMPTY_IMPL_REGEX="\\{[[:space:]]*}$"
ITEM_KINDS=("module" "use" "const" "static" "type/impl" "free function")
IMPL_ITEM_KINDS=("" "" "associated const" "" "associated type" "associated function")

compute_item_rank() {
    local item_line="$1"
    item_rank=""
    has_visibility_order=true
    if [[ $item_line =~ $MOD_REGEX ]]; then
        item_rank=0
    elif [[ $item_line =~ $USE_REGEX ]]; then
        item_rank=1
    elif [[ $item_line =~ $FUNCTION_REGEX ]]; then
        item_rank=5 # avoids misclassifying `const fn` items
    elif [[ $item_line =~ $CONST_REGEX ]]; then
        item_rank=2
    elif [[ $item_line =~ $STATIC_REGEX ]]; then
        item_rank=3
    elif [[ $item_line =~ $TYPE_REGEX || $item_line =~ $IMPL_REGEX ]]; then
        item_rank=4
        if [[ $item_line =~ $IMPL_REGEX ]]; then
            has_visibility_order=false
        fi
    fi
}

exit_code=0
while read -r -d '' file; do
    highest_rank=-1
    highest_visibility_ranks=(-1 -1 -1 -1 -1 -1)
    is_in_impl=false
    impl_highest_rank=-1
    impl_highest_visibility_ranks=(-1 -1 -1 -1 -1 -1)
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $is_in_impl == true && $line =~ $IMPL_END_REGEX ]]; then
            is_in_impl=false
            continue
        fi
        item_line="$line"
        if [[ $is_in_impl == true ]]; then
            item_line="${line#"    "}"
        fi
        compute_item_rank "$item_line"
        if [[ -z $item_rank ]]; then
            continue
        fi
        if [[ $is_in_impl == true ]]; then
            if ((item_rank < impl_highest_rank)); then
                echo "$file:$line_number: ${IMPL_ITEM_KINDS[item_rank]} items should be defined before ${IMPL_ITEM_KINDS[impl_highest_rank]} items"
                exit_code=1
            elif ((item_rank > impl_highest_rank)); then
                impl_highest_rank=$item_rank
            fi
        else
            if ((item_rank < highest_rank)); then
                echo "$file:$line_number: ${ITEM_KINDS[item_rank]} items should be defined before ${ITEM_KINDS[highest_rank]} items"
                exit_code=1
            elif ((item_rank > highest_rank)); then
                highest_rank=$item_rank
            fi
        fi
        if [[ $has_visibility_order == true ]]; then
            visibility_rank=$(compute_visibility_rank "$item_line")
            if [[ $is_in_impl == true ]]; then
                highest_visibility_rank=${impl_highest_visibility_ranks[item_rank]}
                if ((visibility_rank < highest_visibility_rank)); then
                    echo "$file:$line_number: ${VISIBILITY_KINDS[visibility_rank]} ${IMPL_ITEM_KINDS[item_rank]} items should be defined before ${VISIBILITY_KINDS[highest_visibility_rank]} ${IMPL_ITEM_KINDS[item_rank]} items"
                    exit_code=1
                elif ((visibility_rank > highest_visibility_rank)); then
                    impl_highest_visibility_ranks[item_rank]=$visibility_rank
                fi
            else
                highest_visibility_rank=${highest_visibility_ranks[item_rank]}
                if ((visibility_rank < highest_visibility_rank)); then
                    echo "$file:$line_number: ${VISIBILITY_KINDS[visibility_rank]} ${ITEM_KINDS[item_rank]} items should be defined before ${VISIBILITY_KINDS[highest_visibility_rank]} ${ITEM_KINDS[item_rank]} items"
                    exit_code=1
                elif ((visibility_rank > highest_visibility_rank)); then
                    highest_visibility_ranks[item_rank]=$visibility_rank
                fi
            fi
        fi
        if [[ $is_in_impl == false && $item_line =~ $IMPL_REGEX && ! $item_line =~ $EMPTY_IMPL_REGEX ]]; then
            is_in_impl=true
            impl_highest_rank=-1
            impl_highest_visibility_ranks=(-1 -1 -1 -1 -1 -1)
        fi
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit "$exit_code"
