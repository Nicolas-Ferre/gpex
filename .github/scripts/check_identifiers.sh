#!/bin/bash
set -euo pipefail
# shellcheck disable=SC1091
source "$(dirname "$0")/config.sh"

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

LIFETIME_REGEX="\'([A-Za-z_][A-Za-z0-9_]*)[^']"
LET_REGEX="let[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]"
LET_MUT_REGEX="let[[:space:]]mut[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]"
CONSTANT_REGEX="const[[:space:]]([A-Za-z_][A-Za-z0-9_]*):"
STATIC_REGEX="static[[:space:]]([A-Za-z_][A-Za-z0-9_]*):"
# Only lower case to avoid matching generic types, because one-letter generic types are allowed:
PARAMETER_REGEX="[(,][[:space:]]?([a-z_][a-z0-9_]*):"
FUNCTION_REGEX="fn[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[<(]"
STRUCT_REGEX="struct[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[<;{(]"
FIELD_REGEX="^[[:space:]]*([A-Za-z_][A-Za-z0-9_]*):"
PUB_FIELD_REGEX="^[[:space:]]*pub[[:space:]]([A-Za-z_][A-Za-z0-9_]*):"
PUB_MOD_FIELD_REGEX="^[[:space:]]*pub\([^)]+\)[[:space:]]([A-Za-z_][A-Za-z0-9_]*):"
ENUM_REGEX="enum[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[<{]"
UNION_REGEX="union[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[<{]"
VARIANT_REGEX="^[[:space:]]*([A-Za-z_][A-Za-z0-9_]*),$"
TRAIT_REGEX="trait[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[<{]"
TYPE_REGEX="type[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[=;]"
MODULE_REGEX="mod[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[;{]"
MACRO_REGEX="macro_rules![[:space:]]([A-Za-z_][A-Za-z0-9_]*)"
# Only lower case to avoid matching generic types, because one-letter generic types are allowed:
BINDING_REGEX="[(,|{][[:space:]]?([a-z_][a-z0-9_]*)[[:space:]]?[\),\|}]"
FOR_LOOP_VARIABLE_REGEX="for[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]"

is_comment_line() {
    [[ "$line" =~ ^[[:space:]]*// ]]
}

check_underscore_variables() {
    if [[ "$line" =~ "let _ = " ]]; then
        show_error "\`let _ = <value>;\` is used instead of \`_ = <value>;\`"
    fi
}

check_lifetime_plural() {
    local remaining="$line"
    while [[ $remaining =~ $LIFETIME_REGEX ]]; do
        remaining="${remaining#*"${BASH_REMATCH[0]}"}"
        local lifetime="${BASH_REMATCH[1]}"
        if [[ $lifetime =~ s$ ]]; then
            show_error "\`$lifetime\` lifetime name should not be plural"
        fi
    done
}

check_identifier() {
    local name_regex="$1"
    local identifier_type="$2"
    local check_single_letter="$3"
    local remaining="$line"
    while [[ $remaining =~ $name_regex ]]; do
        remaining="${remaining#*"${BASH_REMATCH[0]}"}"
        local identifier=${BASH_REMATCH[1]}
        if [[ $check_single_letter == "true" && ${#identifier} -eq 1 && $identifier != "_" ]]; then
            show_error "\`$identifier\` $identifier_type has too short name"
        fi
        split_identifier=' '$(echo "$identifier" |
            sed -E 's/([a-z])([A-Z])/\1 \2/g' |
            tr '_-' ' ' |
            tr '[:upper:]' '[:lower:]')' '
        for word in "${FORBIDDEN_WORDS[@]}"; do
            if [[ $split_identifier =~ [[:space:]]"$word"[[:space:]] ]]; then
                show_error "\`$identifier\` $identifier_type contains forbidden word '$word'"
            fi
            if [[ $split_identifier =~ [[:space:]]"$word"s[[:space:]] ]]; then
                show_error "\`$identifier\` $identifier_type contains forbidden word '${word}s'"
            fi
        done
    done
}

show_error() {
    local message="$1"
    echo "$file:$line_number: $message"
    exit_code=1
}

exit_code=0

while read -r -d '' file; do
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if is_comment_line; then
            continue
        fi
        check_single_letter=$([[ $file =~ gpex$ ]] && echo "false" || echo "true")
        check_underscore_variables
        check_lifetime_plural
        check_identifier "$LIFETIME_REGEX" "lifetime" "$check_single_letter"
        check_identifier "$LET_REGEX" "variable" "$check_single_letter"
        check_identifier "$LET_MUT_REGEX" "variable" "$check_single_letter"
        check_identifier "$CONSTANT_REGEX" "constant" "$check_single_letter"
        check_identifier "$STATIC_REGEX" "static variable" "$check_single_letter"
        check_identifier "$PARAMETER_REGEX" "parameter" "$check_single_letter"
        check_identifier "$FUNCTION_REGEX" "function" "$check_single_letter"
        check_identifier "$STRUCT_REGEX" "struct" "$check_single_letter"
        check_identifier "$FIELD_REGEX" "field" "$check_single_letter"
        check_identifier "$PUB_FIELD_REGEX" "field" "$check_single_letter"
        check_identifier "$PUB_MOD_FIELD_REGEX" "field" "$check_single_letter"
        check_identifier "$ENUM_REGEX" "enum" "$check_single_letter"
        check_identifier "$UNION_REGEX" "union" "$check_single_letter"
        check_identifier "$VARIANT_REGEX" "variant" "$check_single_letter"
        check_identifier "$TRAIT_REGEX" "trait" "$check_single_letter"
        check_identifier "$TYPE_REGEX" "type alias" "$check_single_letter"
        check_identifier "$MODULE_REGEX" "module" "$check_single_letter"
        check_identifier "$MACRO_REGEX" "macro" "$check_single_letter"
        check_identifier "$BINDING_REGEX" "binding variable" "$check_single_letter"
        check_identifier "$FOR_LOOP_VARIABLE_REGEX" "for loop variable" "$check_single_letter"
    done <"$file"
done < <(find src/ tests/ \( -name "*.rs" -o -name "*.gpex" \) -type f -print0)

exit "$exit_code"
