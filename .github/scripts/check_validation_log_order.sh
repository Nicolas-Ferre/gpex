#!/bin/bash
set -euo pipefail

FUNCTION_START_REGEX='^(pub(\([^)]*\))?[[:space:]]+)?fn[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*)'
LOG_LEVEL_REGEX='LogLevel::(Error|Warning)'
SCOPED_VISIBILITY_REGEX='^pub\(([^)]*)\)'

get_function_visibility() {
    if [[ $line == pub\ * ]]; then
        function_visibility=pub
    elif [[ $line =~ $SCOPED_VISIBILITY_REGEX ]]; then
        function_visibility="pub(${BASH_REMATCH[1]})"
    else
        function_visibility=private
    fi
}

check_function() {
    if [[ -z $function_level ]]; then
        return
    fi
    if [[ $function_visibility != "$current_visibility" ]]; then
        current_visibility=$function_visibility
        has_warning=false
    fi
    if [[ $function_level == Warning ]]; then
        has_warning=true
    elif [[ $has_warning == true ]]; then
        echo "$file:$function_start_line: error log function \`$function_name\` should be defined before warning log functions with $function_visibility visibility"
        exit_code=1
    fi
}

exit_code=0
while read -r -d '' file; do
    line_number=0
    function_name=""
    function_visibility=""
    function_level=""
    function_start_line=0
    current_visibility=""
    has_warning=false
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $line =~ $FUNCTION_START_REGEX ]]; then
            next_function_name=${BASH_REMATCH[3]}
            check_function
            function_name=$next_function_name
            get_function_visibility
            function_level=""
            function_start_line=$line_number
        elif [[ -n $function_name && -z $function_level && $line =~ $LOG_LEVEL_REGEX ]]; then
            function_level=${BASH_REMATCH[1]}
        fi
    done <"$file"
    check_function
done < <(find src/compiler/validation/logs/ -type f -name "*.rs" -print0)

exit "$exit_code"
