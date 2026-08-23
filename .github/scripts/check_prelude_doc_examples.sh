#!/bin/bash
set -euo pipefail

check_docstring() {
    if ((docstring_start_line > 0)) && [[ $has_examples == false ]]; then
        echo "$file:$docstring_start_line: prelude docstring should include a \`# Examples\` section"
        exit_code=1
    fi
    docstring_start_line=0
    has_examples=false
}

exit_code=0
while read -r -d '' file; do
    line_number=0
    docstring_start_line=0
    has_examples=false
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $line == "///"* ]]; then
            if ((docstring_start_line == 0)); then
                docstring_start_line=$line_number
            fi
            if [[ $line == "/// # Examples" ]]; then
                has_examples=true
            fi
        else
            check_docstring
        fi
    done <"$file"
    check_docstring
done < <(find prelude/ -type f -name "*.gpex" -print0)

exit "$exit_code"
