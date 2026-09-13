#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

ARCHITECTURE_DOC_PATH="doc/architecture.md"
CRATE_ROOT_PATH="src/lib.rs"
IDENT="[A-Za-z_][A-Za-z0-9_]*"
CRATE_ROOT_PATH_REGEX="crate::($IDENT)([^a-zA-Z0-9_]|$)"
USE_CRATE_GROUP_START_REGEX="^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?use[[:space:]]+crate::[[:space:]]*\\{"
PUB_USE_START_REGEX="^[[:space:]]*pub(\\([^)]*\\))?[[:space:]]+use[[:space:]]+"
REEXPORT_LEAF_REGEX="($IDENT)[[:space:]]*(as[[:space:]]+($IDENT))?[[:space:]]*(,|}|;)"
MERMAID_EDGE_REGEX="^[[:space:]]*($IDENT)[[:space:]]*-->[[:space:]]*(\\|[^|]*\\|[[:space:]]*)?($IDENT)[[:space:]]*(%%.*)?$"
COUPLING_HEADING_REGEX='^##[[:space:]]+Module[[:space:]]+coupling[[:space:]]*$'
H2_REGEX='^##[[:space:]]'
H3_REGEX='^###[[:space:]]'
SRC_HEADING="### \`src/\`"

code_edges=()
code_edge_files=()
doc_edges=()
src_modules=()
reexport_names=()
reexport_modules=()
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

add_crate_target() {
    local source_module="$1"
    local target="$2"
    local file="$3"
    local resolved="$target"
    if ! in_array "$target" "${src_modules[@]-}"; then
        resolved="$(reexport_module "$target")"
        if [[ -z $resolved ]]; then
            return
        fi
    fi
    if in_array "$resolved" "${src_modules[@]-}" && [[ $resolved != "$source_module" ]]; then
        add_code_edge "$source_module --> $resolved" "$file"
    fi
}

add_doc_edge() {
    local edge="$1"
    if in_array "$edge" "${doc_edges[@]-}"; then
        return
    fi
    doc_edges+=("$edge")
}

add_reexports_from_use() {
    local statement="$1"
    local origin
    local remaining
    local ident
    local alias
    local leaf
    if [[ ! $statement =~ use[[:space:]]+($IDENT) ]]; then
        return
    fi
    origin="${BASH_REMATCH[1]}"
    if ! in_array "$origin" "${src_modules[@]-}"; then
        return
    fi
    remaining="${statement#*"${BASH_REMATCH[0]}"}"
    while [[ $remaining =~ $REEXPORT_LEAF_REGEX ]]; do
        ident="${BASH_REMATCH[1]}"
        alias="${BASH_REMATCH[3]}"
        remaining="${remaining#*"${BASH_REMATCH[0]}"}"
        leaf="${alias:-$ident}"
        if [[ $leaf == self || $leaf == super || $leaf == crate ]] ||
            in_array "$leaf" "${reexport_names[@]-}"; then
            continue
        fi
        reexport_names+=("$leaf")
        reexport_modules+=("$origin")
    done
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

reexport_module() {
    local searched="$1"
    local i
    for i in "${!reexport_names[@]}"; do
        if [[ ${reexport_names[$i]} == "$searched" ]]; then
            printf '%s\n' "${reexport_modules[$i]}"
            return
        fi
    done
}

collect_crate_group_targets() {
    local statement="$1"
    local file_path="$2"
    local source_module="$3"
    local remaining
    local ident
    local brace_depth=0
    local is_at_path_start=true
    if [[ ! $statement =~ crate::[[:space:]]*\{ ]]; then
        return
    fi
    remaining="${statement#*"${BASH_REMATCH[0]}"}"
    brace_depth=1
    while [[ -n $remaining ]]; do
        if [[ $remaining =~ ^[[:space:]]+ ]]; then
            remaining="${remaining#"${BASH_REMATCH[0]}"}"
            continue
        fi
        if [[ $remaining =~ ^\{ ]]; then
            brace_depth=$((brace_depth + 1))
            remaining="${remaining#\{}"
            is_at_path_start=true
            continue
        fi
        if [[ $remaining =~ ^\} ]]; then
            brace_depth=$((brace_depth - 1))
            remaining="${remaining#\}}"
            is_at_path_start=false
            if ((brace_depth == 0)); then
                return
            fi
            continue
        fi
        if [[ $remaining =~ ^, ]]; then
            remaining="${remaining#,}"
            if ((brace_depth == 1)); then
                is_at_path_start=true
            fi
            continue
        fi
        if [[ $remaining =~ ^:: ]]; then
            remaining="${remaining#::}"
            is_at_path_start=false
            continue
        fi
        if [[ $is_at_path_start == false && $remaining =~ ^as[[:space:]]+ ]]; then
            remaining="${remaining#"${BASH_REMATCH[0]}"}"
            if [[ $remaining =~ ^$IDENT ]]; then
                remaining="${remaining#"${BASH_REMATCH[0]}"}"
            fi
            continue
        fi
        if [[ $remaining =~ ^$IDENT ]]; then
            ident="${BASH_REMATCH[0]}"
            remaining="${remaining#"$ident"}"
            if ((brace_depth == 1)) && [[ $is_at_path_start == true ]]; then
                add_crate_target "$source_module" "$ident" "$file_path"
            fi
            is_at_path_start=false
            continue
        fi
        remaining="${remaining:1}"
        is_at_path_start=false
    done
}

collect_file_code_edges() {
    local file_path="$1"
    local report_file="$1"
    local source_module="${file_path#src/}"
    local line
    local remaining
    local target
    local is_in_use_crate_group=false
    local use_statement=""
    source_module="${source_module%%/*}"
    while IFS= read -r line || [[ -n $line ]]; do
        if [[ $line =~ ^[[:space:]]*// ]]; then
            continue
        fi
        remaining="$line"
        while [[ $remaining =~ $CRATE_ROOT_PATH_REGEX ]]; do
            target="${BASH_REMATCH[1]}"
            remaining="${remaining#*"${BASH_REMATCH[0]}"}"
            add_crate_target "$source_module" "$target" "$report_file"
        done
        if [[ $is_in_use_crate_group == false && $line =~ $USE_CRATE_GROUP_START_REGEX ]]; then
            is_in_use_crate_group=true
            use_statement="$line"
        elif [[ $is_in_use_crate_group == true ]]; then
            use_statement+=" $line"
        fi
        if [[ $is_in_use_crate_group == true && $line == *';' ]]; then
            collect_crate_group_targets "$use_statement" "$report_file" "$source_module"
            is_in_use_crate_group=false
            use_statement=""
        fi
    done <"$file_path"
}

collect_src_modules() {
    local dir
    while IFS= read -r -d '' dir; do
        src_modules+=("$(basename "$dir")")
    done < <(find src -mindepth 1 -maxdepth 1 -type d -print0)
}

collect_crate_root_reexports() {
    local line
    local is_in_use=false
    local use_statement=""
    if [[ ! -f $CRATE_ROOT_PATH ]]; then
        echo "$CRATE_ROOT_PATH: file not found"
        exit_code=1
        return
    fi
    while IFS= read -r line || [[ -n $line ]]; do
        if [[ $line =~ ^[[:space:]]*// ]]; then
            continue
        fi
        if [[ $is_in_use == false && $line =~ $PUB_USE_START_REGEX ]]; then
            is_in_use=true
            use_statement="$line"
        elif [[ $is_in_use == true ]]; then
            use_statement+=" $line"
        fi
        if [[ $is_in_use == true && $line == *';' ]]; then
            add_reexports_from_use "$use_statement"
            is_in_use=false
            use_statement=""
        fi
    done <"$CRATE_ROOT_PATH"
}

collect_code_edges() {
    local file_path
    while read -r -d '' file_path; do
        collect_file_code_edges "$file_path"
    done < <(find src -mindepth 2 -type f -name "*.rs" -print0)
}

collect_doc_edges() {
    local line
    local is_in_coupling=false
    local is_in_src=false
    local is_in_mermaid=false
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
            is_in_coupling=true
            is_in_src=false
            is_in_mermaid=false
            continue
        fi
        if [[ $is_in_coupling == true && $line =~ $H2_REGEX ]]; then
            is_in_coupling=false
            is_in_src=false
            is_in_mermaid=false
            continue
        fi
        if [[ $is_in_coupling == true && $line == "$SRC_HEADING" ]]; then
            has_src_heading=true
            is_in_src=true
            is_in_mermaid=false
            continue
        fi
        if [[ $is_in_coupling == true && $line =~ $H3_REGEX ]]; then
            is_in_src=false
            is_in_mermaid=false
            continue
        fi
        if [[ $is_in_src == true && $line == '```mermaid' ]]; then
            has_mermaid_diagram=true
            is_in_mermaid=true
            continue
        fi
        if [[ $is_in_mermaid == true && $line == '```' ]]; then
            is_in_mermaid=false
            continue
        fi
        if [[ $is_in_mermaid == true && $line =~ $MERMAID_EDGE_REGEX ]]; then
            from="${BASH_REMATCH[1]}"
            to="${BASH_REMATCH[3]}"
            add_doc_edge "$from --> $to"
        fi
    done <"$ARCHITECTURE_DOC_PATH"
    if [[ $has_coupling_heading == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing \`## Module coupling\` section"
        exit_code=1
    elif [[ $has_src_heading == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing $SRC_HEADING heading in the module coupling section"
        exit_code=1
    elif [[ $has_mermaid_diagram == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing mermaid diagram under $SRC_HEADING"
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
collect_crate_root_reexports
collect_code_edges
collect_doc_edges
if [[ $has_mermaid_diagram == true ]]; then
    compare_edges
fi
exit "$exit_code"
