#!/bin/bash
set -euo pipefail

FUNCTION_START_REGEX='^(pub[[:space:]]+)?(const[[:space:]]+)?fn[[:space:]]'
EXPECTED_RESULT_REGEX='[^[:space:]].*//[[:space:]]*[^[:space:]]'

reset_docstring() {
    is_in_docstring=false
    is_in_gpex_block=false
    has_expected_result=false
    missing_result_lines=()
    missing_result_count=0
}

exit_code=0
while read -r -d '' file; do
    line_number=0
    reset_docstring
    while IFS= read -r line; do
        line_number=$((line_number + 1))
        if [[ $line == "///"* ]]; then
            if [[ $is_in_docstring == false ]]; then
                reset_docstring
                is_in_docstring=true
            fi
            doc_line=${line#"///"}
            doc_line=${doc_line#" "}
            if [[ $doc_line == '```gpex' ]]; then
                is_in_gpex_block=true
                has_expected_result=false
                gpex_block_start_line=$line_number
            elif [[ $is_in_gpex_block == true && $doc_line == '```' ]]; then
                if [[ $has_expected_result == false ]]; then
                    missing_result_lines+=("$gpex_block_start_line")
                    missing_result_count=$((missing_result_count + 1))
                fi
                is_in_gpex_block=false
            elif [[ $is_in_gpex_block == true && $doc_line =~ $EXPECTED_RESULT_REGEX ]]; then
                has_expected_result=true
            fi
            continue
        fi
        if [[ $line =~ $FUNCTION_START_REGEX ]]; then
            if ((missing_result_count > 0)); then
                for missing_result_line in "${missing_result_lines[@]}"; do
                    echo "$file:$missing_result_line: prelude function \`gpex\` example should include an inline expected-result comment"
                    exit_code=1
                done
            fi
        fi
        reset_docstring
    done <"$file"
done < <(find prelude/ -type f -name "*.gpex" -print0)

exit "$exit_code"
