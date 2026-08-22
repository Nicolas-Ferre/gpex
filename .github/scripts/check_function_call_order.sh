#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

IGNORED_FUNCTIONS=(
    "new"   # very common function name
    "iter"  # very common function name
    "parse" # very common function name for parsing pass
)
EXCLUSION_REGEX="no-fn-check"
IMPL_BLOCK_START_REGEX="^impl([[:space:]<]|$)"
TRAIT_IMPL_BLOCK_START_REGEX="^impl.*[[:space:]]for[[:space:]]"
IMPL_BLOCK_END_REGEX="^}"
EMPTY_IMPL_BLOCK_REGEX="^impl.*\{\}[[:space:]]*$"
FUNCTION_DEFINITION_START_REGEX="^([[:space:]]*)(.*)fn[[:space:]]([a-zA-Z0-9_]+)[^;]*$"
FUNCTION_END="}"
FUNCTION_CALL_REGEX="([a-zA-Z_][a-zA-Z0-9_]*)[<(]"

check_function_call() {
    local regex="$1"
    local remaining="$line"
    while [[ $remaining =~ $regex ]]; do
        remaining="${remaining#*"${BASH_REMATCH[0]}"}"
        local function_name="${BASH_REMATCH[1]}"
        if is_current_function_pub; then
            if is_function_defined "$function_name" && ! in_array "$function_name" "${IGNORED_FUNCTIONS[@]-}"; then
                echo "$file:$line_number: \`$function_name()\` function should be defined after call"
                exit_code=1
            fi
        else
            if is_private_function_defined "$function_name" && ! in_array "$function_name" "${IGNORED_FUNCTIONS[@]-}"; then
                echo "$file:$line_number: \`$function_name()\` function should be defined after call"
                exit_code=1
            fi
        fi

    done
}

is_function_defined() {
    local function_name="$1"
    in_array "$function_name" "${defined_pub_functions[@]-}" ||
        in_array "$function_name" "${defined_priv_functions[@]-}" ||
        is_current_method_defined "$function_name"
}

is_private_function_defined() {
    local function_name="$1"
    in_array "$function_name" "${defined_priv_functions[@]-}" ||
        is_current_private_method_defined "$function_name"
}

is_current_method_defined() {
    [[ $is_current_function_in_impl == true && $is_current_function_in_trait_impl == false ]] &&
        (in_array "$1" "${defined_pub_methods[@]-}" || in_array "$1" "${defined_priv_methods[@]-}")
}

is_current_private_method_defined() {
    [[ $is_current_function_in_impl == true && $is_current_function_in_trait_impl == false ]] &&
        in_array "$1" "${defined_priv_methods[@]-}"
}

is_current_function_pub() {
    [[ $current_function_visibility =~ ^pub.* ]]
}

exit_code=0

while read -r -d '' file; do
    defined_pub_functions=()
    defined_priv_functions=()
    defined_pub_methods=()
    defined_priv_methods=()
    is_in_impl_block=false
    is_in_trait_impl_block=false
    line_number=0
    current_function_name=""
    current_function_indent=""
    current_function_visibility=""
    is_current_function_in_impl=false
    is_current_function_in_trait_impl=false
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $line =~ $EXCLUSION_REGEX ]]; then
            continue
        elif [[ $line =~ $FUNCTION_DEFINITION_START_REGEX ]]; then
            current_function_name="${BASH_REMATCH[3]}"
            current_function_visibility=${BASH_REMATCH[2]}
            current_function_indent=${BASH_REMATCH[1]}
            is_current_function_in_impl=$is_in_impl_block
            is_current_function_in_trait_impl=$is_in_trait_impl_block
        elif [[ -n $current_function_name && $line == "$current_function_indent$FUNCTION_END" ]]; then
            # trait impl blocks are defined before type impl blocks, so trait functions are not registered to avoid false positives
            if [[ $is_current_function_in_impl == false ]]; then
                if is_current_function_pub; then
                    defined_pub_functions+=("$current_function_name")
                else
                    defined_priv_functions+=("$current_function_name")
                fi
            elif [[ $is_current_function_in_trait_impl == false ]]; then
                if is_current_function_pub; then
                    defined_pub_methods+=("$current_function_name")
                else
                    defined_priv_methods+=("$current_function_name")
                fi
            fi
            current_function_name=""
            current_function_indent=""
            current_function_visibility=""
            is_current_function_in_impl=false
            is_current_function_in_trait_impl=false
        elif [[ -n $current_function_name ]]; then
            check_function_call "$FUNCTION_CALL_REGEX"
        fi
        if [[ $line =~ $IMPL_BLOCK_START_REGEX ]]; then
            is_in_impl_block=true
            defined_pub_methods=()
            defined_priv_methods=()
        fi
        if [[ $line =~ $TRAIT_IMPL_BLOCK_START_REGEX ]]; then
            is_in_trait_impl_block=true
        elif [[ $line =~ $IMPL_BLOCK_END_REGEX ]]; then
            is_in_impl_block=false
            is_in_trait_impl_block=false
        fi
        if [[ $line =~ $EMPTY_IMPL_BLOCK_REGEX ]]; then
            is_in_impl_block=false
            is_in_trait_impl_block=false
        fi
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit $exit_code
