#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

ARCHITECTURE_DOC_PATH="doc/architecture.md"
CRATE_ROOT_PATH_REGEX='crate::([A-Za-z_][A-Za-z0-9_]*)([^a-zA-Z0-9_]|$)'
MERMAID_EDGE_REGEX='^[[:space:]]*([A-Za-z_][A-Za-z0-9_]*)[[:space:]]+-->[[:space:]]+([A-Za-z_][A-Za-z0-9_]*)[[:space:]]*$'
COUPLING_HEADING_REGEX='^##[[:space:]]+Module[[:space:]]+coupling[[:space:]]*$'
H2_REGEX='^##[[:space:]]'
H3_REGEX='^###[[:space:]]'
SRC_HEADING="### \`src/\`"

code_edges=()
code_edge_files=()
doc_edges=()
src_modules=()
exit_code=0
has_coupling_heading=false
has_src_heading=false
has_mermaid_diagram=false

add_code_edge() {
    local edge="$1"
    local file="$2"
    if in_array "$edge" "${code_edges[@]-}"; then
        return
    fi
    code_edges+=("$edge")
    code_edge_files+=("$file")
}

add_doc_edge() {
    local edge="$1"
    if in_array "$edge" "${doc_edges[@]-}"; then
        return
    fi
    doc_edges+=("$edge")
}

code_edge_file() {
    local searched="$1"
    local i
    for i in "${!code_edges[@]}"; do
        if [[ ${code_edges[$i]} == "$searched" ]]; then
            printf '%s\n' "${code_edge_files[$i]}"
            return
        fi
    done
}

collect_file_code_edges() {
    local file_path="$1"
    local source_module="${file_path#src/}"
    local line
    local remaining
    local target
    source_module="${source_module%%/*}"
    while IFS= read -r line || [[ -n $line ]]; do
        if [[ $line =~ ^[[:space:]]*// ]]; then
            continue
        fi
        remaining="$line"
        while [[ $remaining =~ $CRATE_ROOT_PATH_REGEX ]]; do
            target="${BASH_REMATCH[1]}"
            remaining="${remaining#*"${BASH_REMATCH[0]}"}"
            if in_array "$target" "${src_modules[@]-}" && [[ $target != "$source_module" ]]; then
                add_code_edge "$source_module --> $target" "$1"
            fi
        done
    done <"$file_path"
}

collect_src_modules() {
    local dir
    while IFS= read -r -d '' dir; do
        src_modules+=("$(basename "$dir")")
    done < <(find src -mindepth 1 -maxdepth 1 -type d -print0)
}

collect_code_edges() {
    local file_path
    while read -r -d '' file_path; do
        collect_file_code_edges "$file_path"
    done < <(find src -mindepth 2 -type f -name "*.rs" -print0)
}

collect_doc_edges() {
    local line
    local in_coupling=false
    local in_src=false
    local in_mermaid=false
    local from
    local to
    if [[ ! -f $ARCHITECTURE_DOC_PATH ]]; then
        echo "$ARCHITECTURE_DOC_PATH: file not found"
        exit_code=1
        return
    fi
    while IFS= read -r line || [[ -n $line ]]; do
        if [[ $line =~ $COUPLING_HEADING_REGEX ]]; then
            has_coupling_heading=true
            in_coupling=true
            in_src=false
            in_mermaid=false
            continue
        fi
        if [[ $in_coupling == true && $line =~ $H2_REGEX ]]; then
            in_coupling=false
            in_src=false
            in_mermaid=false
            continue
        fi
        if [[ $in_coupling == true && $line == "$SRC_HEADING" ]]; then
            has_src_heading=true
            in_src=true
            in_mermaid=false
            continue
        fi
        if [[ $in_coupling == true && $line =~ $H3_REGEX ]]; then
            in_src=false
            in_mermaid=false
            continue
        fi
        if [[ $in_src == true && $line == '```mermaid' ]]; then
            has_mermaid_diagram=true
            in_mermaid=true
            continue
        fi
        if [[ $in_mermaid == true && $line == '```' ]]; then
            in_mermaid=false
            continue
        fi
        if [[ $in_mermaid == true && $line =~ $MERMAID_EDGE_REGEX ]]; then
            from="${BASH_REMATCH[1]}"
            to="${BASH_REMATCH[2]}"
            add_doc_edge "$from --> $to"
        fi
    done <"$ARCHITECTURE_DOC_PATH"
    if [[ $has_coupling_heading == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing \`## Module coupling\` section"
        exit_code=1
    elif [[ $has_src_heading == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing \`$SRC_HEADING\` heading in the module coupling section"
        exit_code=1
    elif [[ $has_mermaid_diagram == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing mermaid diagram under \`$SRC_HEADING\`"
        exit_code=1
    fi
}

compare_edges() {
    local edge
    for edge in "${code_edges[@]-}"; do
        if ! in_array "$edge" "${doc_edges[@]-}"; then
            echo "$ARCHITECTURE_DOC_PATH: missing coupling edge \`$edge\` (e.g. $(code_edge_file "$edge"))"
            exit_code=1
        fi
    done
    for edge in "${doc_edges[@]-}"; do
        if ! in_array "$edge" "${code_edges[@]-}"; then
            echo "$ARCHITECTURE_DOC_PATH: extra coupling edge \`$edge\`"
            exit_code=1
        fi
    done
}

collect_src_modules
collect_code_edges
collect_doc_edges
if [[ $has_mermaid_diagram == true ]]; then
    compare_edges
fi
exit "$exit_code"
