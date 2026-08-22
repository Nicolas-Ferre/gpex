#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

VISIBILITY_REGEX='(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?'
MODIFIER_REGEX='(async[[:space:]]+|unsafe[[:space:]]+|const[[:space:]]+|default[[:space:]]+|extern[[:space:]]+)*'
ITEM_START_REGEX="^[[:space:]]*${VISIBILITY_REGEX}${MODIFIER_REGEX}(fn|struct|enum|trait|impl|mod|const|static|type|union)([[:space:]]|<)|^[[:space:]]*macro_rules![[:space:]]*!|^[[:space:]]*macro[[:space:]]"
USE_START_REGEX="^[[:space:]]*${VISIBILITY_REGEX}use[[:space:]]"
INLINE_MOD_START_REGEX="^[[:space:]]*${VISIBILITY_REGEX}mod[[:space:]]+[a-zA-Z0-9_]+[[:space:]]*;"
CONST_ITEM_REGEX="^[[:space:]]*${VISIBILITY_REGEX}const[[:space:]]+[a-zA-Z_]"
CONST_FN_ITEM_REGEX="^[[:space:]]*${VISIBILITY_REGEX}const[[:space:]]+fn[[:space:]]"
TYPE_ITEM_REGEX="^[[:space:]]*${VISIBILITY_REGEX}type[[:space:]]+[a-zA-Z_]"
PREAMBLE_REGEX="^[[:space:]]*(#\[|///|//!|//)"
EMPTY_REGEX="^[[:space:]]*$"
FUNCTION_START_REGEX="^([[:space:]]*).*fn[[:space:]][a-zA-Z0-9_]+[^\;]*$"

reset_preamble() {
    pending_preamble_line=0
    pending_preamble_indent=""
    preamble_previous_line=""
}

find_last_item() {
    local item_index item
    last_item_index=-1
    last_item_kind=""

    for item_index in "${!last_items[@]}"; do
        item=${last_items[$item_index]}
        if [[ ${item%%:*} == "$current_scope" ]]; then
            last_item_index=$item_index
            last_item_kind=${item#*:}
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

exit_code=0
while read -r -d '' file; do
    block_stack=()
    last_items=()
    next_block_id=0
    pending_preamble_line=0
    pending_preamble_indent=""
    preamble_previous_line=""
    line_number=0
    previous_line=""

    while IFS= read -r line; do
        line_number=$((line_number + 1))
        indent="${line%%[^[:space:]]*}"

        if ((${#block_stack[@]} > 0)); then
            last_block=$((${#block_stack[@]} - 1))
            last_block_indent=${block_stack[$last_block]%%:*}
            if [[ $line == "${last_block_indent}}" ]]; then
                block_stack=("${block_stack[@]:0:$last_block}")
            fi
        fi

        if ((${#block_stack[@]} > 0)); then
            current_scope=${block_stack[$((${#block_stack[@]} - 1))]##*:}
            current_block=${block_stack[$((${#block_stack[@]} - 1))]#*:}
        else
            current_scope="file"
            current_block=""
        fi

        if [[ $current_block == fn:* ]]; then
            reset_preamble
        elif [[ $line =~ $PREAMBLE_REGEX ]]; then
            if ((pending_preamble_line == 0)); then
                pending_preamble_line=$line_number
                pending_preamble_indent=$indent
                preamble_previous_line=$previous_line
            fi
        elif [[ $line =~ $ITEM_START_REGEX ]] && ! [[ $line =~ $USE_START_REGEX ]] &&
            ! [[ $line =~ $INLINE_MOD_START_REGEX ]]; then
            separator_line=$previous_line
            if ((pending_preamble_line > 0)) && [[ $line == "$pending_preamble_indent"* ]]; then
                separator_line=$preamble_previous_line
            fi

            current_item_kind=$(item_kind)
            find_last_item || true
            if ((last_item_index >= 0)) && [[ $current_item_kind == "$last_item_kind" &&
                ($current_item_kind == const || $current_item_kind == type) ]]; then
                spacing_exception=true
            else
                spacing_exception=false
            fi

            if ((last_item_index >= 0)) && [[ $spacing_exception == false ]] &&
                ! [[ $separator_line =~ $EMPTY_REGEX ]]; then
                violation_line=$line_number
                if ((pending_preamble_line > 0)); then
                    violation_line=$pending_preamble_line
                fi
                echo "$file:$violation_line: items should be separated by an empty line"
                exit_code=1
            fi

            if ((last_item_index >= 0)); then
                last_items[last_item_index]="$current_scope:$current_item_kind"
            else
                last_items+=("$current_scope:$current_item_kind")
            fi
            reset_preamble

            if [[ $line == *"{"* ]]; then
                block_kind=item
                if [[ $line =~ $FUNCTION_START_REGEX ]]; then
                    block_kind=fn
                fi
                block_stack+=("$indent:$block_kind:$next_block_id")
                next_block_id=$((next_block_id + 1))
            fi
        elif ! [[ $line =~ $EMPTY_REGEX ]] && [[ $line != "$pending_preamble_indent"* ]]; then
            reset_preamble
        fi

        previous_line=$line
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit "$exit_code"
