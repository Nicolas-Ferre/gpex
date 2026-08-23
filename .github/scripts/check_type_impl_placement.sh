#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

VISIBILITY_REGEX='(pub(\([^)]*\))?[[:space:]]+)?'
TYPE_DEFINITION_REGEX="^${VISIBILITY_REGEX}(struct|enum|union)[[:space:]]+([A-Z][a-zA-Z0-9_]*)"
IMPL_START_REGEX='^(unsafe[[:space:]]+)?impl([[:space:]<]|$)'
OTHER_ITEM_START_REGEX="^${VISIBILITY_REGEX}(mod|use|const|static|type|trait|fn|async|unsafe|extern|macro)([[:space:]!(<]|$)"
MACRO_CALL_REGEX='^([a-zA-Z_][a-zA-Z0-9_]*::)*[a-zA-Z_][a-zA-Z0-9_]*!'

extract_impl_type() {
    local impl_line="$1"
    local previous_line=""
    while [[ $impl_line != "$previous_line" ]]; do
        previous_line="$impl_line"
        impl_line=$(printf '%s\n' "$impl_line" | sed -E 's/<[^<>]*>//g')
    done
    printf '%s\n' "$impl_line" | grep -Eo '[A-Z][a-zA-Z0-9_]*' | tail -n 1 || true
}

collect_defined_types() {
    defined_types=()
    while read -r -d '' file; do
        while IFS= read -r line; do
            if [[ $line =~ $TYPE_DEFINITION_REGEX ]] &&
                ! in_array "${BASH_REMATCH[4]}" "${defined_types[@]-}"; then
                defined_types+=("${BASH_REMATCH[4]}")
            fi
        done <"$file"
    done < <(find src/ tests/ -type f -name "*.rs" -print0)
}

check_file() {
    local file_path="$1"
    local current_type=""
    local line_number=0
    local line
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $line =~ $TYPE_DEFINITION_REGEX ]]; then
            current_type="${BASH_REMATCH[4]}"
        elif [[ $line =~ $IMPL_START_REGEX ]]; then
            local impl_type
            impl_type=$(extract_impl_type "$line")
            if in_array "$impl_type" "${defined_types[@]-}" && [[ $impl_type != "$current_type" ]]; then
                echo "$file_path:$line_number: \`$impl_type\` impl block should be defined immediately after its type definition"
                exit_code=1
            fi
            current_type="$impl_type"
        elif [[ $line =~ $OTHER_ITEM_START_REGEX || $line =~ $MACRO_CALL_REGEX ]]; then
            current_type=""
        fi
    done <"$file_path"
}

collect_defined_types
exit_code=0
while read -r -d '' file; do
    check_file "$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)
exit "$exit_code"
