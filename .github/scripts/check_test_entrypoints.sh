#!/bin/bash
set -euo pipefail

# shellcheck disable=SC1091
source "$(dirname "$0")/config.sh"
# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

exit_code=0
while read -r -d '' test_dir_path; do
    if in_array "$test_dir_path" "${EXCLUDED_TEST_DIR_PATHS[@]-}"; then
        continue
    fi
    if ! find "$test_dir_path" -maxdepth 1 -type f -name "test_*.gpex" -print -quit |
        grep -q .; then
        echo "$test_dir_path: test directory should contain at least one test_*.gpex file"
        exit_code=1
    fi
done < <(find tests/integration/ -type d \
    \( -name "ok_*" -o -name "wgsl_*" -o -name "nok_*" \) -print0)
exit "$exit_code"
