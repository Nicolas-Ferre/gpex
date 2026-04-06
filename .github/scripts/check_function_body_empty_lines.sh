#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

FUNCTION_START_REGEX="^([[:space:]]*).*fn[[:space:]][a-zA-Z0-9_]+[^\;]*$"
FUNCTION_END="}"
EMPTY_REGEX="^[[:space:]]*$"

exit_code=0
while read -r -d '' file; do
    is_in_function=false
    function_indent=""
    line_number=0
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $line =~ $FUNCTION_START_REGEX ]]; then
            is_in_function=true
            function_indent="${BASH_REMATCH[1]}"
        elif [[ $is_in_function == true && $line == "$function_indent$FUNCTION_END" ]]; then
            is_in_function=false
            function_indent=""
        elif [[ $is_in_function == true && $line =~ $EMPTY_REGEX ]]; then
            echo "$file:$line_number: empty lines are not allowed in function bodies"
            exit_code=1
        fi
    done <"$file"
done < <(find src/ tests/ -type f -name "*.rs" -print0)
exit $exit_code
