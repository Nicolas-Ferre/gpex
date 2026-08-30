#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/config.sh"

LIFETIME_REGEX="\'([A-Za-z_][A-Za-z0-9_]*)[^']"
LET_REGEX="let[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]"
LET_MUT_REGEX="let[[:space:]]mut[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]"
CONSTANT_REGEX="const[[:space:]]([A-Za-z_][A-Za-z0-9_]*):"
STATIC_REGEX="static[[:space:]]([A-Za-z_][A-Za-z0-9_]*):"
# Only lower case to avoid matching generic types:
PARAMETER_REGEX="[(,][[:space:]]?([a-z_][a-z0-9_]*):"
FUNCTION_REGEX="fn[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[<(]"
STRUCT_REGEX="struct[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[<;{(]"
FIELD_REGEX="^[[:space:]]*([A-Za-z_][A-Za-z0-9_]*):"
PUB_FIELD_REGEX="^[[:space:]]*pub[[:space:]]([A-Za-z_][A-Za-z0-9_]*):"
PUB_MOD_FIELD_REGEX="^[[:space:]]*pub\([^)]+\)[[:space:]]([A-Za-z_][A-Za-z0-9_]*):"
ENUM_REGEX="enum[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[<{]"
UNION_REGEX="union[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[<{]"
VARIANT_REGEX="^[[:space:]]*([A-Za-z_][A-Za-z0-9_]*)(\([^)]*\))?,$"
TRAIT_REGEX="trait[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[<{]"
TYPE_REGEX="type[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[=;]"
MODULE_REGEX="mod[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]?[;{]"
MACRO_REGEX="macro_rules![[:space:]]([A-Za-z_][A-Za-z0-9_]*)"
GENERIC_DEFINITION_REGEX="(struct|enum|union|trait|type|fn)[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*<|impl[[:space:]]*<"
# Only lower case to avoid matching generic types:
BINDING_REGEX="[(,|{][[:space:]]?([a-z_][a-z0-9_]*)[[:space:]]?[),|}]"
FOR_LOOP_VARIABLE_REGEX="for[[:space:]]([A-Za-z_][A-Za-z0-9_]*)[[:space:]]"
UPPERCASE_LETTERS="ABCDEFGHIJKLMNOPQRSTUVWXYZ"
LOWERCASE_LETTERS="abcdefghijklmnopqrstuvwxyz"

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

split_identifier() {
    local identifier="$1"
    local split_identifier=" "
    local previous_character=""
    local character_index
    for ((character_index = 0; character_index < ${#identifier}; character_index++)); do
        local character="${identifier:character_index:1}"
        if [[ $previous_character =~ [a-z] && $character =~ [A-Z] ]]; then
            split_identifier+=" "
        fi
        case "$character" in
        [_-]) split_identifier+=" " ;;
        *)
            if [[ $UPPERCASE_LETTERS == *"$character"* ]]; then
                local uppercase_prefix="${UPPERCASE_LETTERS%%"$character"*}"
                split_identifier+="${LOWERCASE_LETTERS:${#uppercase_prefix}:1}"
            else
                split_identifier+="$character"
            fi
            ;;
        esac
        previous_character="$character"
    done
    SPLIT_IDENTIFIER="$split_identifier "
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
        split_identifier "$identifier"
        for word in "${FORBIDDEN_WORDS[@]}"; do
            if [[ $SPLIT_IDENTIFIER =~ [[:space:]]"$word"[[:space:]] ]]; then
                show_error "\`$identifier\` $identifier_type contains forbidden word '$word'"
            fi
            if [[ $SPLIT_IDENTIFIER =~ [[:space:]]"$word"s[[:space:]] ]]; then
                show_error "\`$identifier\` $identifier_type contains forbidden word '${word}s'"
            fi
        done
    done
}

check_generic_type_param() {
    local param="$1"
    param="${param#"${param%%[![:space:]]*}"}"
    if [[ $param =~ ^([A-Za-z_][A-Za-z0-9_]*) ]]; then
        local identifier=${BASH_REMATCH[1]}
        if [[ ${#identifier} -eq 1 && $identifier != "_" ]]; then
            show_error "\`$identifier\` generic type has too short name"
        fi
    fi
}

check_generic_type_params() {
    local params="$1"
    local param=""
    local depth=0
    local character_index
    for ((character_index = 0; character_index < ${#params}; character_index++)); do
        local character="${params:character_index:1}"
        case "$character" in
        '<')
            depth=$((depth + 1))
            param+="$character"
            ;;
        '>')
            depth=$((depth - 1))
            param+="$character"
            ;;
        ',')
            if ((depth == 0)); then
                check_generic_type_param "$param"
                param=""
            else
                param+=","
            fi
            ;;
        *) param+="$character" ;;
        esac
    done
    check_generic_type_param "$param"
}

read_generic_definition() {
    local remaining="$1"
    local character_index
    for ((character_index = 0; character_index < ${#remaining}; character_index++)); do
        local character="${remaining:character_index:1}"
        case "$character" in
        '<')
            generic_definition_depth=$((generic_definition_depth + 1))
            generic_definition_params+="$character"
            ;;
        '>')
            generic_definition_depth=$((generic_definition_depth - 1))
            if ((generic_definition_depth == 0)); then
                check_generic_type_params "$generic_definition_params"
                is_generic_definition=false
                return
            fi
            generic_definition_params+="$character"
            ;;
        *) generic_definition_params+="$character" ;;
        esac
    done
    generic_definition_params+=" "
}

show_error() {
    local message="$1"
    echo "$file:$line_number: $message"
    exit_code=1
}

exit_code=0
while read -r -d '' file; do
    if [[ $file == *.gpex ]]; then
        check_single_letter=false
    else
        check_single_letter=true
    fi
    is_generic_definition=false
    generic_definition_depth=0
    generic_definition_params=""
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if is_comment_line; then
            continue
        fi
        if [[ $is_generic_definition == "true" ]]; then
            read_generic_definition "$line"
        elif [[ $line =~ $GENERIC_DEFINITION_REGEX ]]; then
            is_generic_definition=true
            generic_definition_depth=1
            generic_definition_params=""
            read_generic_definition "${line#*"${BASH_REMATCH[0]}"}"
        fi
        check_underscore_variables
        if [[ $line == *"'"* ]]; then
            check_lifetime_plural
            check_identifier "$LIFETIME_REGEX" "lifetime" "$check_single_letter"
        fi
        if [[ $line == *"let"* ]]; then
            check_identifier "$LET_REGEX" "variable" "$check_single_letter"
            check_identifier "$LET_MUT_REGEX" "variable" "$check_single_letter"
        fi
        if [[ $line == *"const"* ]]; then
            check_identifier "$CONSTANT_REGEX" "constant" "$check_single_letter"
        fi
        if [[ $line == *"static"* ]]; then
            check_identifier "$STATIC_REGEX" "static variable" "$check_single_letter"
        fi
        if [[ $line == *"("* || $line == *","* ]]; then
            check_identifier "$PARAMETER_REGEX" "parameter" "$check_single_letter"
        fi
        if [[ $line == *"fn"* ]]; then
            check_identifier "$FUNCTION_REGEX" "function" "$check_single_letter"
        fi
        if [[ $line == *"struct"* ]]; then
            check_identifier "$STRUCT_REGEX" "struct" "$check_single_letter"
        fi
        if [[ $line == *":"* ]]; then
            check_identifier "$FIELD_REGEX" "field" "$check_single_letter"
            check_identifier "$PUB_FIELD_REGEX" "field" "$check_single_letter"
            check_identifier "$PUB_MOD_FIELD_REGEX" "field" "$check_single_letter"
        fi
        if [[ $line == *"enum"* ]]; then
            check_identifier "$ENUM_REGEX" "enum" "$check_single_letter"
        fi
        if [[ $line == *"union"* ]]; then
            check_identifier "$UNION_REGEX" "union" "$check_single_letter"
        fi
        if [[ $line == *, ]]; then
            check_identifier "$VARIANT_REGEX" "variant" "$check_single_letter"
        fi
        if [[ $line == *"trait"* ]]; then
            check_identifier "$TRAIT_REGEX" "trait" "$check_single_letter"
        fi
        if [[ $line == *"type"* ]]; then
            check_identifier "$TYPE_REGEX" "type alias" "$check_single_letter"
        fi
        if [[ $line == *"mod"* ]]; then
            check_identifier "$MODULE_REGEX" "module" "$check_single_letter"
        fi
        if [[ $line == *"macro_rules!"* ]]; then
            check_identifier "$MACRO_REGEX" "macro" "$check_single_letter"
        fi
        if [[ $line == *"("* || $line == *","* || $line == *"|"* || $line == *"{"* ]]; then
            check_identifier "$BINDING_REGEX" "binding variable" "$check_single_letter"
        fi
        if [[ $line == *"for"* ]]; then
            check_identifier "$FOR_LOOP_VARIABLE_REGEX" "for loop variable" "$check_single_letter"
        fi
    done <"$file"
done < <(find src/ tests/ \( -name "*.rs" -o -name "*.gpex" \) -type f -print0)
exit "$exit_code"
