#!/bin/bash
set -euo pipefail

# It is considered that the analyzed code is compiling and formatted with Rustfmt.

# shellcheck disable=SC1091
source "$(dirname "$0")/utils.sh"

ARCHITECTURE_DOC_PATH="doc/architecture.md"
CRATE_ROOT_PATH="src/lib.rs"
IDENT="[A-Za-z_][A-Za-z0-9_]*"
USE_CRATE_START_REGEX="^[[:space:]]*(pub(\\([^)]*\\))?[[:space:]]+)?use[[:space:]]+crate::"
PUB_USE_START_REGEX="^[[:space:]]*pub(\\([^)]*\\))?[[:space:]]+use[[:space:]]+"
REEXPORT_LEAF_REGEX="($IDENT)[[:space:]]*(as[[:space:]]+($IDENT))?[[:space:]]*(,|}|;)"
MERMAID_EDGE_REGEX="^[[:space:]]*($IDENT)[[:space:]]*-->[[:space:]]*(\\|[^|]*\\|[[:space:]]*)?($IDENT)[[:space:]]*(%%.*)?$"
MERMAID_NODE_REGEX="^[[:space:]]*($IDENT)[[:space:]]*(%%.*)?$"
MERMAID_SUBGRAPH_REGEX="^[[:space:]]*subgraph[[:space:]]+($IDENT)"
COUPLING_HEADING_REGEX='^##[[:space:]]+Module[[:space:]]+coupling[[:space:]]*$'
H2_REGEX='^##[[:space:]]'

code_edges=()
code_edge_files=()
doc_edges=()
doc_nodes=()
src_modules=()
scope_module_keys=()
expanded_scopes=()
reexport_names=()
reexport_modules=()
collected_modules=()
tree_remaining=""
current_file=""
exit_code=0
has_coupling_heading=false
has_mermaid_diagram=false

edge_key() {
    local scope="$1"
    local edge="$2"
    printf '%s\t%s\n' "$scope" "$edge"
}

display_edge() {
    local key="$1"
    local scope="${key%%	*}"
    local edge="${key#*	}"
    if [[ -z $scope ]]; then
        printf '%s\n' "$edge"
    else
        printf '%s (subgraph %s)\n' "$edge" "$scope"
    fi
}

add_code_edge() {
    local scope="$1"
    local from="$2"
    local to="$3"
    local file="$4"
    local key
    key="$(edge_key "$scope" "$from --> $to")"
    if in_array "$key" "${code_edges[@]-}"; then
        return
    fi
    code_edges+=("$key")
    code_edge_files+=("$file")
}

add_doc_edge() {
    local scope="$1"
    local from="$2"
    local to="$3"
    local key
    key="$(edge_key "$scope" "$from --> $to")"
    if in_array "$key" "${doc_edges[@]-}"; then
        return
    fi
    doc_edges+=("$key")
}

add_doc_node() {
    local scope="$1"
    local name="$2"
    local key
    key="$(edge_key "$scope" "$name")"
    if in_array "$key" "${doc_nodes[@]-}"; then
        return
    fi
    doc_nodes+=("$key")
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

collect_child_modules() {
    local parent_dir="$1"
    local path
    local base
    collected_modules=()
    while IFS= read -r -d '' path; do
        base="$(basename "$path")"
        if [[ -d $path ]]; then
            collected_modules+=("$base")
            continue
        fi
        case "$base" in
        mod.rs | lib.rs | main.rs) continue ;;
        *.rs) collected_modules+=("${base%.rs}") ;;
        esac
    done < <(find "$parent_dir" -mindepth 1 -maxdepth 1 \( -type d -o -name '*.rs' \) -print0)
}

register_scope_modules() {
    local scope="$1"
    local dir
    local module
    if [[ -z $scope ]]; then
        dir="src"
    else
        dir="src/$scope"
    fi
    collect_child_modules "$dir"
    for module in "${collected_modules[@]-}"; do
        scope_module_keys+=("$(edge_key "$scope" "$module")")
    done
}

is_module_in_scope() {
    local scope="$1"
    local name="$2"
    in_array "$(edge_key "$scope" "$name")" "${scope_module_keys[@]-}"
}

scope_source() {
    local scope="$1"
    local file_path="$2"
    local rel="${file_path#src/}"
    local rest
    local child
    if [[ -z $scope ]]; then
        printf '%s\n' "${rel%%/*}"
        return
    fi
    case "$rel" in
    "$scope"/mod.rs) return ;;
    "$scope"/*)
        rest="${rel#"$scope"/}"
        child="${rest%%/*}"
        printf '%s\n' "${child%.rs}"
        ;;
    esac
}

current_scope() {
    local scope=""
    local part
    for part in "$@"; do
        if [[ -z $scope ]]; then
            scope="$part"
        else
            scope="$scope/$part"
        fi
    done
    printf '%s\n' "$scope"
}

skip_tree_space() {
    if [[ $tree_remaining =~ ^[[:space:]]+ ]]; then
        tree_remaining="${tree_remaining#"${BASH_REMATCH[0]}"}"
    fi
}

skip_tree_as_alias() {
    skip_tree_space
    if [[ $tree_remaining =~ ^as[[:space:]]+ ]]; then
        tree_remaining="${tree_remaining#"${BASH_REMATCH[0]}"}"
        skip_tree_space
        if [[ $tree_remaining =~ ^$IDENT ]]; then
            tree_remaining="${tree_remaining#"${BASH_REMATCH[0]}"}"
        fi
    fi
}

add_crate_path() {
    local slash_path="$1"
    local first="${slash_path%%/*}"
    local resolved="$first"
    local src_source
    local scope
    local source
    local target
    local depth
    local remaining_scope
    local segments=()
    if ! in_array "$first" "${src_modules[@]-}"; then
        resolved="$(reexport_module "$first")"
        if [[ -z $resolved ]]; then
            return
        fi
    fi
    src_source="$(scope_source "" "$current_file")"
    if in_array "$resolved" "${src_modules[@]-}" && [[ $resolved != "$src_source" ]]; then
        add_code_edge "" "$src_source" "$resolved" "$current_file"
    fi
    IFS=/ read -ra segments <<<"$slash_path"
    for scope in "${expanded_scopes[@]-}"; do
        source="$(scope_source "$scope" "$current_file")"
        if [[ -z $source ]]; then
            continue
        fi
        if [[ $slash_path != "$scope" && $slash_path != "$scope"/* ]]; then
            continue
        fi
        depth=1
        remaining_scope="$scope"
        while [[ $remaining_scope == */* ]]; do
            remaining_scope="${remaining_scope#*/}"
            depth=$((depth + 1))
        done
        if ((${#segments[@]} <= depth)); then
            continue
        fi
        target="${segments[$depth]}"
        if ! is_module_in_scope "$scope" "$target" || [[ $target == "$source" ]]; then
            continue
        fi
        add_code_edge "$scope" "$source" "$target" "$current_file"
    done
}

parse_use_tree_item() {
    local prefix="$1"
    local ident
    local path
    skip_tree_space
    if [[ $tree_remaining =~ ^\{ ]]; then
        tree_remaining="${tree_remaining#\{}"
        while true; do
            skip_tree_space
            if [[ -z $tree_remaining || $tree_remaining =~ ^\} ]]; then
                tree_remaining="${tree_remaining#\}}"
                return
            fi
            if [[ $tree_remaining =~ ^, ]]; then
                tree_remaining="${tree_remaining#,}"
                continue
            fi
            parse_use_tree_item "$prefix"
        done
    fi
    if [[ ! $tree_remaining =~ ^$IDENT ]]; then
        return
    fi
    ident="${BASH_REMATCH[0]}"
    tree_remaining="${tree_remaining#"$ident"}"
    if [[ $ident == self || $ident == super || $ident == crate ]]; then
        if [[ -n $prefix ]]; then
            add_crate_path "$prefix"
        fi
        skip_tree_as_alias
        return
    fi
    if [[ -z $prefix ]]; then
        path="$ident"
    else
        path="$prefix/$ident"
    fi
    skip_tree_space
    if [[ $tree_remaining =~ ^:: ]]; then
        tree_remaining="${tree_remaining#::}"
        parse_use_tree_item "$path"
        return
    fi
    skip_tree_as_alias
    add_crate_path "$path"
}

scan_crate_paths() {
    local text="$1"
    tree_remaining="$text"
    while [[ $tree_remaining =~ crate:: ]]; do
        tree_remaining="${tree_remaining#*crate::}"
        parse_use_tree_item ""
    done
}

collect_file_code_edges() {
    local file_path="$1"
    local line
    local is_in_use_crate=false
    local use_statement=""
    current_file="$file_path"
    while IFS= read -r line || [[ -n $line ]]; do
        if [[ $line =~ ^[[:space:]]*// ]]; then
            continue
        fi
        if [[ $is_in_use_crate == false && $line =~ $USE_CRATE_START_REGEX ]]; then
            is_in_use_crate=true
            use_statement="$line"
        elif [[ $is_in_use_crate == true ]]; then
            use_statement+=" $line"
        else
            scan_crate_paths "$line"
        fi
        if [[ $is_in_use_crate == true && $line == *';' ]]; then
            scan_crate_paths "$use_statement"
            is_in_use_crate=false
            use_statement=""
        fi
    done <"$file_path"
}

collect_src_modules() {
    collect_child_modules "src"
    src_modules=("${collected_modules[@]-}")
    register_scope_modules ""
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

collect_doc() {
    local line
    local is_in_coupling=false
    local is_in_mermaid=false
    local from
    local to
    local subgraph_stack=()
    local new_stack
    local stack_index
    local scope
    local parent_scope
    local subgraph
    if [[ ! -f $ARCHITECTURE_DOC_PATH ]]; then
        echo "$ARCHITECTURE_DOC_PATH: file not found"
        exit_code=1
        return
    fi
    while IFS= read -r line || [[ -n $line ]]; do
        if [[ $line =~ $COUPLING_HEADING_REGEX ]]; then
            has_coupling_heading=true
            is_in_coupling=true
            is_in_mermaid=false
            subgraph_stack=()
            continue
        fi
        if [[ $is_in_coupling == true && $line =~ $H2_REGEX ]]; then
            is_in_coupling=false
            is_in_mermaid=false
            subgraph_stack=()
            continue
        fi
        if [[ $is_in_coupling == true && $line == '```mermaid' ]]; then
            has_mermaid_diagram=true
            is_in_mermaid=true
            subgraph_stack=()
            continue
        fi
        if [[ $is_in_mermaid == true && $line == '```' ]]; then
            is_in_mermaid=false
            subgraph_stack=()
            continue
        fi
        if [[ $is_in_mermaid != true ]]; then
            continue
        fi
        if [[ $line =~ $MERMAID_SUBGRAPH_REGEX ]]; then
            subgraph="${BASH_REMATCH[1]}"
            parent_scope="$(current_scope "${subgraph_stack[@]-}")"
            if ! is_module_in_scope "$parent_scope" "$subgraph"; then
                if [[ -z $parent_scope ]]; then
                    echo "$ARCHITECTURE_DOC_PATH: subgraph \`$subgraph\` is not a crate-root module"
                else
                    echo "$ARCHITECTURE_DOC_PATH: subgraph \`$subgraph\` is not a child of \`$parent_scope\`"
                fi
                exit_code=1
            else
                add_doc_node "$parent_scope" "$subgraph"
                subgraph_stack+=("$subgraph")
                scope="$(current_scope "${subgraph_stack[@]}")"
                if ! in_array "$scope" "${expanded_scopes[@]-}"; then
                    expanded_scopes+=("$scope")
                    register_scope_modules "$scope"
                fi
            fi
            continue
        fi
        if [[ $line =~ ^[[:space:]]*end[[:space:]]*(%%.*)?$ ]]; then
            if ((${#subgraph_stack[@]} > 0)); then
                new_stack=()
                for ((stack_index = 0; stack_index < ${#subgraph_stack[@]} - 1; stack_index++)); do
                    new_stack+=("${subgraph_stack[stack_index]}")
                done
                subgraph_stack=("${new_stack[@]-}")
            fi
            continue
        fi
        scope="$(current_scope "${subgraph_stack[@]-}")"
        if [[ $line =~ $MERMAID_EDGE_REGEX ]]; then
            from="${BASH_REMATCH[1]}"
            to="${BASH_REMATCH[3]}"
            add_doc_node "$scope" "$from"
            add_doc_node "$scope" "$to"
            add_doc_edge "$scope" "$from" "$to"
            continue
        fi
        if [[ $line =~ ^[[:space:]]*(graph|flowchart)[[:space:]] ]]; then
            continue
        fi
        if [[ $line =~ $MERMAID_NODE_REGEX ]]; then
            add_doc_node "$scope" "${BASH_REMATCH[1]}"
        fi
    done <"$ARCHITECTURE_DOC_PATH"
    if [[ $has_coupling_heading == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing \`## Module coupling\` section"
        exit_code=1
    elif [[ $has_mermaid_diagram == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing mermaid diagram in the module coupling section"
        exit_code=1
    fi
}

compare_edges() {
    local edge
    for edge in "${code_edges[@]-}"; do
        if ! in_array "$edge" "${doc_edges[@]-}"; then
            echo "$ARCHITECTURE_DOC_PATH: missing coupling edge \`$(display_edge "$edge")\` (e.g. $(code_edge_file "$edge"))"
            exit_code=1
        fi
    done
    for edge in "${doc_edges[@]-}"; do
        if ! in_array "$edge" "${code_edges[@]-}"; then
            echo "$ARCHITECTURE_DOC_PATH: extra coupling edge \`$(display_edge "$edge")\`"
            exit_code=1
        fi
    done
}

compare_nodes() {
    local key
    local scope
    local name
    for key in "${scope_module_keys[@]-}"; do
        if ! in_array "$key" "${doc_nodes[@]-}"; then
            scope="${key%%	*}"
            name="${key#*	}"
            if [[ -z $scope ]]; then
                echo "$ARCHITECTURE_DOC_PATH: missing coupling node \`$name\`"
            else
                echo "$ARCHITECTURE_DOC_PATH: missing coupling node \`$name\` (subgraph $scope)"
            fi
            exit_code=1
        fi
    done
    for key in "${doc_nodes[@]-}"; do
        if ! in_array "$key" "${scope_module_keys[@]-}"; then
            scope="${key%%	*}"
            name="${key#*	}"
            if [[ -z $scope ]]; then
                echo "$ARCHITECTURE_DOC_PATH: extra coupling node \`$name\`"
            else
                echo "$ARCHITECTURE_DOC_PATH: extra coupling node \`$name\` (subgraph $scope)"
            fi
            exit_code=1
        fi
    done
}

collect_src_modules
collect_crate_root_reexports
collect_doc
collect_code_edges
if [[ $has_mermaid_diagram == true ]]; then
    compare_edges
    compare_nodes
fi
exit "$exit_code"
