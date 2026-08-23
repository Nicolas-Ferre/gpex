#!/bin/bash
set -euo pipefail

FUNCTION_START_REGEX='^(pub[[:space:]]+)?(const[[:space:]]+)?fn[[:space:]]'
BOOL_INTRINSIC_END_REGEX='->[[:space:]]*bool[[:space:]]*=[[:space:]]*intrinsic;'

check_function() {
    if [[ $function_signature =~ $BOOL_INTRINSIC_END_REGEX ]] &&
        [[ $function_first_prose_line != "Returns whether "* ]]; then
        echo "$file:$function_start_line: boolean-returning intrinsic function docstring should start with \`Returns whether ...\`"
        exit_code=1
    fi
    function_signature=""
    function_first_prose_line=""
    function_start_line=0
}

exit_code=0
while read -r -d '' file; do
    line_number=0
    is_in_docstring=false
    pending_first_prose_line=""
    function_signature=""
    function_first_prose_line=""
    function_start_line=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ -n $function_signature ]]; then
            function_signature+=" $line"
            if [[ $line == *";"* ]]; then
                check_function
            fi
            continue
        fi
        if [[ $line == "///"* ]]; then
            if [[ $is_in_docstring == false ]]; then
                pending_first_prose_line=""
                is_in_docstring=true
            fi
            prose_line=${line#"///"}
            prose_line=${prose_line#" "}
            if [[ -z $pending_first_prose_line && -n $prose_line ]]; then
                pending_first_prose_line=$prose_line
            fi
            continue
        fi
        if [[ $line =~ $FUNCTION_START_REGEX ]]; then
            function_signature=$line
            function_first_prose_line=$pending_first_prose_line
            function_start_line=$line_number
            if [[ $line == *";"* ]]; then
                check_function
            fi
        fi
        is_in_docstring=false
        pending_first_prose_line=""
    done <"$file"
done < <(find prelude/ -type f -name "*.gpex" -print0)

exit "$exit_code"
