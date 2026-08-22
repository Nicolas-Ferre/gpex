#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

ITEM_START_REGEX="^[[:space:]]*((pub([[:space:]]*\\([^)]*\\))?[[:space:]]+)?(async[[:space:]]+|unsafe[[:space:]]+|const[[:space:]]+|default[[:space:]]+|extern[[:space:]]+)*(fn|struct|enum|trait|impl|mod|const|static|type|union)([[:space:]]|<)|macro_rules![[:space:]]*!|macro[[:space:]])"
USE_START_REGEX="^[[:space:]]*(pub([[:space:]]*\\([^)]*\\))?[[:space:]]+)?use[[:space:]]"
INLINE_MOD_START_REGEX="^[[:space:]]*(pub([[:space:]]*\\([^)]*\\))?[[:space:]]+)?mod[[:space:]]+[a-zA-Z0-9_]+[[:space:]]*;"
CONST_ITEM_REGEX="^[[:space:]]*(pub([[:space:]]*\\([^)]*\\))?[[:space:]]+)?const[[:space:]]+[a-zA-Z_]"
CONST_FN_ITEM_REGEX="^[[:space:]]*(pub([[:space:]]*\\([^)]*\\))?[[:space:]]+)?const[[:space:]]+fn[[:space:]]"
TYPE_ITEM_REGEX="^[[:space:]]*(pub([[:space:]]*\\([^)]*\\))?[[:space:]]+)?type[[:space:]]+[a-zA-Z_]"
PREAMBLE_REGEX="^[[:space:]]*(#\\[|///|//!|//)"
EMPTY_REGEX="^[[:space:]]*$"
FUNCTION_START_REGEX="^([[:space:]]*).*fn[[:space:]][a-zA-Z0-9_]+[^\;]*$"

is_item_start() {
    [[ $line =~ $ITEM_START_REGEX ]]
}

is_use_start() {
    [[ $line =~ $USE_START_REGEX ]]
}

is_inline_mod_start() {
    [[ $line =~ $INLINE_MOD_START_REGEX ]]
}

is_preamble() {
    [[ $line =~ $PREAMBLE_REGEX ]]
}

is_empty_line() {
    [[ $line =~ $EMPTY_REGEX ]]
}

has_last_item_in_scope() {
    local item_scope
    for item_scope in "${last_item_scopes[@]-}"; do
        if [[ $item_scope == "$current_scope" ]]; then
            return 0
        fi
    done
    return 1
}

item_kind() {
    if [[ $line =~ $CONST_ITEM_REGEX ]] && ! [[ $line =~ $CONST_FN_ITEM_REGEX ]]; then
        printf 'const\n'
    elif [[ $line =~ $TYPE_ITEM_REGEX ]]; then
        printf 'type\n'
    else
        printf 'other\n'
    fi
}

last_item_index_in_scope() {
    local item_index
    for item_index in "${!last_item_scopes[@]}"; do
        if [[ ${last_item_scopes[$item_index]} == "$current_scope" ]]; then
            printf '%s\n' "$item_index"
            return 0
        fi
    done
    printf '%s\n' '-1'
}

exit_code=0
while read -r -d '' file; do
    block_kinds=()
    block_indents=()
    block_ids=()
    last_item_scopes=()
    last_item_kinds=()
    next_block_id=0
    pending_preamble_line=0
    pending_preamble_indent=""
    preamble_previous_line=""
    line_number=0
    previous_line=""

    while IFS= read -r line; do
        line_number=$((line_number + 1))
        indent="${line%%[^[:space:]]*}"

        if ((${#block_indents[@]} > 0)); then
            last_block=$((${#block_indents[@]} - 1))
            if [[ $line == "${block_indents[$last_block]}}" ]]; then
                block_kinds=("${block_kinds[@]:0:$last_block}")
                block_indents=("${block_indents[@]:0:$last_block}")
                block_ids=("${block_ids[@]:0:$last_block}")
            fi
        fi

        if ((${#block_kinds[@]} > 0)) && [[ ${block_kinds[*]} == *fn* ]]; then
            is_in_function=true
        else
            is_in_function=false
        fi

        if ((${#block_ids[@]} > 0)); then
            current_scope="${block_ids[$((${#block_ids[@]} - 1))]}"
        else
            current_scope="file"
        fi

        if [[ $is_in_function == true ]]; then
            pending_preamble_line=0
            pending_preamble_indent=""
            preamble_previous_line=""
        elif is_preamble; then
            if ((pending_preamble_line == 0)); then
                pending_preamble_line=$line_number
                pending_preamble_indent=$indent
                preamble_previous_line=$previous_line
            fi
        elif is_item_start && ! is_use_start && ! is_inline_mod_start; then
            separator_line="$previous_line"
            if ((pending_preamble_line > 0)) && [[ $line == "$pending_preamble_indent"* ]]; then
                separator_line="$preamble_previous_line"
            fi

            current_item_kind=$(item_kind)
            last_item_index=$(last_item_index_in_scope)
            has_item_kind_spacing_exception=false
            if ((last_item_index >= 0)); then
                last_item_kind=${last_item_kinds[$last_item_index]}
                if [[ $current_item_kind == "$last_item_kind" && ($current_item_kind == const || $current_item_kind == type) ]]; then
                    has_item_kind_spacing_exception=true
                fi
            fi

            if has_last_item_in_scope && [[ $has_item_kind_spacing_exception == false ]] && ! [[ $separator_line =~ $EMPTY_REGEX ]]; then
                violation_line=$line_number
                if ((pending_preamble_line > 0)); then
                    violation_line=$pending_preamble_line
                fi
                echo "$file:$violation_line: items should be separated by an empty line"
                exit_code=1
            fi

            if ! has_last_item_in_scope; then
                last_item_scopes+=("$current_scope")
                last_item_kinds+=("$current_item_kind")
            else
                last_item_kinds[last_item_index]="$current_item_kind"
            fi
            pending_preamble_line=0
            pending_preamble_indent=""
            preamble_previous_line=""

            if [[ $line == *"{"* ]]; then
                if [[ $line =~ $FUNCTION_START_REGEX ]]; then
                    block_kinds+=("fn")
                else
                    block_kinds+=("item")
                fi
                block_indents+=("$indent")
                block_ids+=("block-$next_block_id")
                next_block_id=$((next_block_id + 1))
            fi
        elif ! is_empty_line && [[ $line != "$pending_preamble_indent"* ]]; then
            pending_preamble_line=0
            pending_preamble_indent=""
            preamble_previous_line=""
        fi

        previous_line=$line
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit "$exit_code"
