#!/bin/bash
set -euo pipefail

FUNCTION_START_REGEX='^(pub[[:space:]]+)?(const[[:space:]]+)?fn[[:space:]]'
PARAM_NAME_REGEX='([a-zA-Z_][a-zA-Z0-9_]*)[[:space:]]*:'

check_function() {
    local remaining_signature=$function_signature
    while [[ $remaining_signature =~ $PARAM_NAME_REGEX ]]; do
        local param_name=${BASH_REMATCH[1]}
        local param_match=${BASH_REMATCH[0]}
        local param_ref="\`$param_name\`"
        if [[ $function_docstring != *"$param_ref"* ]]; then
            echo "$file:$function_start_line: \`$param_name\` parameter should be referenced in the function docstring"
            exit_code=1
        fi
        remaining_signature=${remaining_signature#*"$param_match"}
    done
    function_signature=""
    function_docstring=""
    function_start_line=0
}

exit_code=0
while read -r -d '' file; do
    line_number=0
    is_in_docstring=false
    pending_docstring=""
    function_signature=""
    function_docstring=""
    function_start_line=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ -n $function_signature ]]; then
            function_signature+=" $line"
            if [[ $line == *";"* || $line == *"{"* ]]; then
                check_function
            fi
            continue
        fi
        if [[ $line == "///"* ]]; then
            if [[ $is_in_docstring == false ]]; then
                pending_docstring=""
                is_in_docstring=true
            fi
            pending_docstring+="$line"$'\n'
            continue
        fi
        if [[ $line =~ $FUNCTION_START_REGEX && $is_in_docstring == true ]]; then
            function_signature=$line
            function_docstring=$pending_docstring
            function_start_line=$line_number
            if [[ $line == *";"* || $line == *"{"* ]]; then
                check_function
            fi
        fi
        is_in_docstring=false
        pending_docstring=""
    done <"$file"
done < <(find prelude/ -type f -name "*.gpex" -print0)

exit "$exit_code"
