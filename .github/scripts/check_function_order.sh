#!/bin/bash
set -euo pipefail
IFS=
# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

IGNORED_FUNCTIONS=(
    "new" # very common function name
)

EXCLUSION_REGEX="fn_check:[[:space:]]off"
FUNCTION_DEFINITION_START_REGEX="^([[:space:]]*).*fn[[:space:]]([a-zA-Z0-9_]+)[^;]*$"
FUNCTION_END="}"
FUNCTION_CALL_REGEX="([a-zA-Z_][a-zA-Z0-9_]*)[<(]"

check_function_call() {
    local regex="$1"
    local remaining="$line"
    while [[ $remaining =~ $regex ]]; do
        local remaining="${remaining#*"${BASH_REMATCH[0]}"}"
        local function_name="${BASH_REMATCH[1]}"
        if is_item_in_array "${defined_functions[@]-}" "$function_name" && ! is_item_in_array "${IGNORED_FUNCTIONS[@]-}" "$function_name"; then
            echo "$file:$line_number: \`$function_name()\` function should be defined after call"
            exit_code=1
        fi
    done
}

exit_code=0

while read -r -d '' file; do
    if [[ $file =~ src/language ]]; then
        continue
    fi
    defined_functions=()
    current_function_name=""
    current_function_indent=""
    line_number=1
    while read -r line; do
        if [[ $line =~ $EXCLUSION_REGEX ]]; then
            continue
        elif [[ $line =~ $FUNCTION_DEFINITION_START_REGEX ]]; then
            current_function_name="${BASH_REMATCH[2]}"
            current_function_indent=${BASH_REMATCH[1]}
        elif [[ -n $current_function_name && $line == "$current_function_indent$FUNCTION_END" ]]; then
            defined_functions+=("$current_function_name")
            current_function_name=""
            current_function_indent=""
        elif [[ -n $current_function_name ]]; then
            check_function_call "$FUNCTION_CALL_REGEX"
        fi
        line_number=$((line_number + 1))
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit $exit_code
