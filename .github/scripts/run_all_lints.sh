#!/bin/bash
set -euo pipefail

is_fast=false
if (($# > 1)); then
    echo "Usage: $0 [--fast]" >&2
    exit 2
fi
if (($# == 1)); then
    if [[ $1 != "--fast" ]]; then
        echo "Usage: $0 [--fast]" >&2
        exit 2
    fi
    is_fast=true
fi

pids=()
commands=()

command_name() {
    local s="$*"
    s="$(printf '%s' "$s" | sed -E 's/[^A-Za-z0-9_-]+/_/g; s/^_+//; s/_+$//')"
    printf '%.50s\n' "$s"
}

start() {
    echo "→ $*"
    (set -xeuo pipefail && "$@") >"target/lints/lint-$(command_name "$*").log" 2>&1 &
    pids+=("$!")
    commands+=("$*")
}

mkdir -p target/lints

cargo fmt -- --check
shfmt --indent 4 -l .github/scripts/*.sh

start git diff --check
start shellcheck .github/scripts/*.sh --shell bash --severity style --external-sources
start bash .github/scripts/check_impl_order.sh
start bash .github/scripts/check_function_body_empty_lines.sh
start bash .github/scripts/check_mod_location.sh
start bash .github/scripts/check_function_call_order.sh
start bash .github/scripts/check_function_visibility_order.sh
start bash .github/scripts/check_item_separation.sh

if [[ $is_fast == false ]]; then
    start bash .github/scripts/check_line_endings.sh
    start bash .github/scripts/check_todos.sh
    start bash .github/scripts/check_file_paths.sh
    start bash .github/scripts/check_identifiers.sh
fi

failed=0
for i in "${!pids[@]}"; do
    command="${commands[$i]}"
    if wait "${pids[$i]}"; then
        echo "✓ $command"
    else
        echo "✗ $command"
        cat "target/lints/lint-$(command_name "$command").log"
        failed=1
    fi
done
exit "$failed"
