#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

FUNCTION_REGEX="^((pub(\([^)]*\))?)[[:space:]]+)?([^[:space:]]+[[:space:]]+)*fn[[:space:]]+([a-zA-Z0-9_]+)"
IMPL_FUNCTION_REGEX="^[[:space:]]*${FUNCTION_REGEX#^}"
IMPL_START_REGEX="^impl([[:space:]]|<)"
IMPL_END_REGEX="^}"
EMPTY_IMPL_REGEX="\\{[[:space:]]*}$"

check_function() {
    local function_line="$1"
    local has_private_function="$2"
    if [[ $function_line =~ ^pub([[:space:]]|\() ]]; then
        if [[ $has_private_function == true ]]; then
            show_error "$function_name"
        fi
    else
        printf -v "$3" "%s" true
    fi
}

show_error() {
    local function_name="$1"
    echo "$file:$line_number: public \`$function_name()\` function should be defined before private functions"
    exit_code=1
}

exit_code=0
while read -r -d '' file; do
    is_in_impl=false
    has_private_root_function=false
    has_private_impl_function=false
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $is_in_impl == true && $line =~ $IMPL_FUNCTION_REGEX ]]; then
            function_name="${BASH_REMATCH[5]}"
            function_line="${line#"    "}"
            check_function "$function_line" "$has_private_impl_function" has_private_impl_function
        elif [[ $is_in_impl == false && $line =~ $FUNCTION_REGEX ]]; then
            function_name="${BASH_REMATCH[5]}"
            check_function "$line" "$has_private_root_function" has_private_root_function
        fi
        if [[ $is_in_impl == false && $line =~ $IMPL_START_REGEX && ! $line =~ $EMPTY_IMPL_REGEX ]]; then
            is_in_impl=true
            has_private_impl_function=false
        elif [[ $is_in_impl == true && $line =~ $IMPL_END_REGEX ]]; then
            is_in_impl=false
        fi
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit "$exit_code"
