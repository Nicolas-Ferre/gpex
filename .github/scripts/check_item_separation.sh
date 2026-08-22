#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

VISIBILITY_REGEX='(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?'
MODIFIER_REGEX='(async[[:space:]]+|unsafe[[:space:]]+|const[[:space:]]+|default[[:space:]]+|extern[[:space:]]+)*'
ITEM_START_REGEX="^[[:space:]]*${VISIBILITY_REGEX}${MODIFIER_REGEX}(fn|struct|enum|trait|impl|const|static|type|union)([[:space:]]|<)|^[[:space:]]*${VISIBILITY_REGEX}mod[[:space:]]+[a-zA-Z0-9_]+[[:space:]]*\{|^[[:space:]]*(macro_rules![[:space:]]+|macro[[:space:]])"
GROUPED_ITEM_REGEX="^[[:space:]]*${VISIBILITY_REGEX}(use[[:space:]]|mod[[:space:]]+[a-zA-Z0-9_]+[[:space:]]*;)"
SPECIAL_ITEM_REGEX="^[[:space:]]*${VISIBILITY_REGEX}(const|type)[[:space:]]+[a-zA-Z_]"
PREAMBLE_REGEX="^[[:space:]]*(#\[|///|//!|//)"
EMPTY_REGEX="^[[:space:]]*$"
FUNCTION_START_REGEX="^[[:space:]]*${VISIBILITY_REGEX}${MODIFIER_REGEX}fn([[:space:]]|<)"

reset_preamble() {
    pending_preamble_line=0
    pending_preamble_indent=""
    preamble_previous_line=""
}

exit_code=0
while read -r -d '' file; do
    last_item_kinds=()
    function_end=""
    pending_function_end=""
    reset_preamble
    line_number=0
    previous_line=""
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        indent="${line%%[^[:space:]]*}"
        indent_length=${#indent}
        if [[ -n $pending_function_end ]]; then
            if [[ $line == *"{"* && $line != *"}"* ]]; then
                function_end=$pending_function_end
            fi
            if [[ $line == *"{"* || $line == *";"* ]]; then
                pending_function_end=""
            fi
            previous_line=$line
            continue
        elif [[ -n $function_end && $line != "$function_end" ]]; then
            previous_line=$line
            continue
        elif [[ -n $function_end ]]; then
            function_end=""
        fi
        if [[ $line =~ $PREAMBLE_REGEX ]]; then
            if ((pending_preamble_line == 0)); then
                pending_preamble_line=$line_number
                pending_preamble_indent=$indent
                preamble_previous_line=$previous_line
            fi
        elif [[ $line =~ $GROUPED_ITEM_REGEX ]]; then
            last_item_kinds[indent_length]=other
            reset_preamble
        elif [[ $line =~ $ITEM_START_REGEX ]]; then
            separator_line=$previous_line
            violation_line=$line_number
            if ((pending_preamble_line > 0)) && [[ $indent == "$pending_preamble_indent" ]]; then
                separator_line=$preamble_previous_line
                violation_line=$pending_preamble_line
            fi
            # A new item owns a fresh child scope at the next Rustfmt indentation level.
            unset 'last_item_kinds[indent_length + 4]'
            current_item_kind=other
            if ! [[ $line =~ $FUNCTION_START_REGEX ]] && [[ $line =~ $SPECIAL_ITEM_REGEX ]]; then
                current_item_kind=${BASH_REMATCH[3]}
            fi
            last_item_kind=${last_item_kinds[$indent_length]-}
            if [[ -n $last_item_kind && ! ($current_item_kind == "$last_item_kind" &&
                ($current_item_kind == const || $current_item_kind == type)) &&
                ! $separator_line =~ $EMPTY_REGEX ]]; then
                echo "$file:$violation_line: items should be separated by an empty line"
                exit_code=1
            fi
            last_item_kinds[indent_length]=$current_item_kind
            reset_preamble

            if [[ $line =~ $FUNCTION_START_REGEX ]]; then
                if [[ $line != *"{"* && $line != *";"* ]]; then
                    pending_function_end="${indent}}"
                elif [[ $line == *"{"* && $line != *"}"* ]]; then
                    function_end="${indent}}"
                fi
            fi
        elif ! [[ $line =~ $EMPTY_REGEX ]] && [[ $line != "$pending_preamble_indent"* ]]; then
            reset_preamble
        fi
        previous_line=$line
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)
exit "$exit_code"
