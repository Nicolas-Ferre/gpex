#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

VISIBILITY_REGEX='(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?'
MODIFIER_REGEX='(async[[:space:]]+|unsafe[[:space:]]+|const[[:space:]]+|default[[:space:]]+|extern[[:space:]]+)*'
ITEM_START_REGEX="^[[:space:]]*${VISIBILITY_REGEX}${MODIFIER_REGEX}(fn|struct|enum|trait|impl|const|static|type|union)([[:space:]]|<)|^[[:space:]]*${VISIBILITY_REGEX}mod[[:space:]]+[a-zA-Z0-9_]+[[:space:]]*\{|^[[:space:]]*(macro_rules![[:space:]]+|macro[[:space:]])"
GROUPED_ITEM_REGEX="^[[:space:]]*${VISIBILITY_REGEX}(use[[:space:]]|mod[[:space:]]+[a-zA-Z0-9_]+[[:space:]]*;)"
CONST_ITEM_REGEX="^[[:space:]]*${VISIBILITY_REGEX}const[[:space:]]+[a-zA-Z_]"
TYPE_ITEM_REGEX="^[[:space:]]*${VISIBILITY_REGEX}type[[:space:]]+[a-zA-Z_]"
PREAMBLE_REGEX="^[[:space:]]*(#\[|///|//!|//)"
EMPTY_REGEX="^[[:space:]]*$"
FUNCTION_START_REGEX="^[[:space:]]*${VISIBILITY_REGEX}${MODIFIER_REGEX}fn([[:space:]]|<)"

reset_preamble() {
    pending_preamble_line=0
    pending_preamble_indent=""
    preamble_previous_line=""
}

reset_file_state() {
    last_item_kinds=()
    function_end=""
    pending_function_end=""
    reset_preamble
    line_number=0
    previous_line=""
}

consume_function_scope_line() {
    if [[ -n $pending_function_end ]]; then
        if [[ $line == *"{"* && $line != *"}"* ]]; then
            function_end=$pending_function_end
        fi
        if [[ $line == *"{"* || $line == *";"* ]]; then
            pending_function_end=""
        fi
        return 0
    fi
    local function_end_regex="^${function_end}[[:space:]]*(//.*)?$"
    if [[ -n $function_end && ! $line =~ $function_end_regex ]]; then
        return 0
    fi
    if [[ -n $function_end ]]; then
        function_end=""
    fi
    return 1
}

record_preamble() {
    if ((pending_preamble_line == 0)); then
        pending_preamble_line=$line_number
        pending_preamble_indent=$indent
        preamble_previous_line=$previous_line
    fi
}

get_item_kind() {
    local item_kind=other
    if [[ $line =~ $GROUPED_ITEM_REGEX ]]; then
        item_kind="grouped"
    elif ! [[ $line =~ $FUNCTION_START_REGEX ]] && [[ $line =~ $CONST_ITEM_REGEX ]]; then
        item_kind="const"
    elif ! [[ $line =~ $FUNCTION_START_REGEX ]] && [[ $line =~ $TYPE_ITEM_REGEX ]]; then
        item_kind="type"
    fi
    printf '%s\n' "$item_kind"
}

check_item_separation() {
    local separator_line=$previous_line
    local violation_line=$line_number
    local current_item_kind
    local last_item_kind
    if ((pending_preamble_line > 0)) && [[ $indent == "$pending_preamble_indent" ]]; then
        separator_line=$preamble_previous_line
        violation_line=$pending_preamble_line
    fi
    # A new item resets tracking for its next nested indentation level.
    unset 'last_item_kinds[indent_length + 4]'
    current_item_kind=$(get_item_kind)
    last_item_kind=${last_item_kinds[$indent_length]-}
    if [[ -n $last_item_kind && ! ($current_item_kind == "$last_item_kind" &&
        ($current_item_kind == const || $current_item_kind == type ||
        $current_item_kind == grouped)) &&
        ! $separator_line =~ $EMPTY_REGEX ]]; then
        echo "$file:$violation_line: items should be separated by an empty line"
        exit_code=1
    fi
    last_item_kinds[indent_length]=$current_item_kind
    reset_preamble
}

start_function_scope() {
    if ! [[ $line =~ $FUNCTION_START_REGEX ]]; then
        return
    fi
    if [[ $line != *"{"* && $line != *";"* ]]; then
        pending_function_end="${indent}}"
    elif [[ $line == *"{"* && $line != *"}"* ]]; then
        function_end="${indent}}"
    fi
}

process_item_line() {
    if [[ $line =~ $PREAMBLE_REGEX ]]; then
        record_preamble
    elif [[ $line =~ $GROUPED_ITEM_REGEX ]]; then
        check_item_separation
    elif [[ $line =~ $ITEM_START_REGEX ]]; then
        check_item_separation
        start_function_scope
    elif ! [[ $line =~ $EMPTY_REGEX ]] && [[ $line != "$pending_preamble_indent"* ]]; then
        reset_preamble
    fi
}

process_line() {
    if consume_function_scope_line; then
        return
    fi
    process_item_line
}

process_file() {
    local file_path="$1"
    file=$file_path
    reset_file_state
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        indent="${line%%[^[:space:]]*}"
        indent_length=${#indent}
        process_line
        previous_line=$line
    done <"$file"
}

process_files() {
    while read -r -d '' file; do
        process_file "$file"
    done < <(find src/ tests/ -type f -name "*.rs" -print0)
}

exit_code=0
process_files
exit "$exit_code"
