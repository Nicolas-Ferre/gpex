#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/config.sh"
# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

USE_START_REGEX='^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?use[[:space:]]+'
FUNCTION_DEFINITION_REGEX='fn[[:space:]]+([a-z_][a-zA-Z0-9_]*)'
IMPORT_LEAF_REGEX='(^|[^a-zA-Z0-9_])([a-z_][a-zA-Z0-9_]*)[[:space:]]*(,|}|;|$)'
QUALIFIED_TYPE_REGEX='(^|[^a-zA-Z0-9_])(((r#)?[a-z_][a-zA-Z0-9_]*::)+[A-Z][a-zA-Z0-9_]*[a-z][a-zA-Z0-9_]*)'

collect_imported_functions() {
    local remaining="$1"
    while [[ $remaining =~ $IMPORT_LEAF_REGEX ]]; do
        remaining="${remaining#*"${BASH_REMATCH[0]}"}"
        local function_name="${BASH_REMATCH[2]}"
        if ! in_array "$function_name" "${imported_functions[@]-}"; then
            imported_functions+=("$function_name")
        fi
    done
}

collect_file_items() {
    imported_functions=()
    local_functions=()
    local is_in_use=false
    local use_statement=""
    while IFS= read -r line; do
        if [[ $is_in_use == false && $line =~ $USE_START_REGEX ]]; then
            is_in_use=true
            use_statement="$line"
        elif [[ $is_in_use == true ]]; then
            use_statement+=" $line"
        elif [[ $line =~ $FUNCTION_DEFINITION_REGEX ]]; then
            local function_name="${BASH_REMATCH[1]}"
            if ! in_array "$function_name" "${local_functions[@]-}"; then
                local_functions+=("$function_name")
            fi
        fi
        if [[ $is_in_use == true && $line == *';' ]]; then
            collect_imported_functions "$use_statement"
            is_in_use=false
            use_statement=""
        fi
    done <"$file"
}

check_function_calls() {
    local function_name
    for function_name in "${imported_functions[@]-}"; do
        if in_array "$function_name" "${local_functions[@]-}" ||
            in_array "$function_name" "${EXCLUDED_FUNCTIONS[@]-}"; then
            continue
        fi
        local function_call_regex="(^|[^a-zA-Z0-9_:.!])${function_name}[[:space:]]*\\("
        if [[ $line =~ $function_call_regex ]]; then
            echo "$file:$line_number: \`$function_name()\` function should be qualified with its parent module"
            exit_code=1
        fi
    done
}

check_qualified_types() {
    local remaining="$line"
    while [[ $remaining =~ $QUALIFIED_TYPE_REGEX ]]; do
        remaining="${remaining#*"${BASH_REMATCH[0]}"}"
        local type_path="${BASH_REMATCH[2]}"
        if ! in_array "$type_path" "${EXCLUDED_TYPE_PATHS[@]-}"; then
            echo "$file:$line_number: \`$type_path\` type should be imported and used without its parent module"
            exit_code=1
        fi
    done
}

exit_code=0
while read -r -d '' file; do
    collect_file_items
    is_in_use=false
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $is_in_use == false && $line =~ $USE_START_REGEX ]]; then
            is_in_use=true
        fi
        if [[ $is_in_use == true ]]; then
            if [[ $line == *';' ]]; then
                is_in_use=false
            fi
            continue
        fi
        if [[ $line =~ ^[[:space:]]*// ]]; then
            continue
        fi
        if [[ ! $line =~ ^[[:space:]]*#\[ && ! $line =~ $FUNCTION_DEFINITION_REGEX ]]; then
            check_function_calls
        fi
        check_qualified_types
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)

exit "$exit_code"
