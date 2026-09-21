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
H3_REGEX='^###[[:space:]]'
TRACKED_HEADING_REGEX="^###[[:space:]]+\`(src(/$IDENT)*)/\`[[:space:]]*$"

code_edges=()
code_edge_files=()
doc_edges=()
doc_nodes=()
src_modules=()
scope_module_keys=()
expanded_scopes=()
tracked_scopes=()
reexport_names=()
reexport_modules=()
collected_modules=()
tree_remaining=""
current_file=""
current_scope=""
is_in_tracked_heading=false
is_in_mermaid=false
heading_has_mermaid=false
exit_code=0
has_coupling_heading=false
has_src_heading=false
has_mermaid_diagram=false

edge_key() {
    local scope="$1"
    local edge="$2"
    printf '%s\t%s\n' "$scope" "$edge"
}

folder_display() {
    local scope="$1"
    if [[ -z $scope ]]; then
        printf 'src/'
    else
        printf 'src/%s/' "$scope"
    fi
}

display_edge() {
    local key="$1"
    local scope="${key%%	*}"
    local edge="${key#*	}"
    if [[ -z $scope ]]; then
        printf '%s\n' "$edge"
    else
        printf "%s (under \`%s\`)\n" "$edge" "$(folder_display "$scope")"
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
        child="${rel%%/*}"
        printf '%s\n' "${child%.rs}"
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
    done < <(find src -type f -name "*.rs" ! -name lib.rs ! -name main.rs -print0)
}

start_tracked_heading() {
    local folder="$1"
    local scope
    local dir="$folder"
    if [[ $folder == src ]]; then
        scope=""
        has_src_heading=true
    else
        scope="${folder#src/}"
        if [[ ! -d $dir ]]; then
            echo "$ARCHITECTURE_DOC_PATH: heading ### \`$folder/\` is not a directory"
            exit_code=1
            return 1
        fi
        if ! in_array "$scope" "${expanded_scopes[@]-}"; then
            expanded_scopes+=("$scope")
            register_scope_modules "$scope"
        fi
    fi
    if ((${#tracked_scopes[@]} > 0)) && in_array "$scope" "${tracked_scopes[@]}"; then
        echo "$ARCHITECTURE_DOC_PATH: duplicated coupling heading ### \`$folder/\`"
        exit_code=1
    else
        tracked_scopes+=("$scope")
    fi
    current_scope="$scope"
    is_in_tracked_heading=true
    heading_has_mermaid=false
}

finish_tracked_heading() {
    local folder
    if [[ $is_in_tracked_heading != true ]]; then
        return
    fi
    if [[ $heading_has_mermaid == false ]]; then
        folder="$(folder_display "$current_scope")"
        echo "$ARCHITECTURE_DOC_PATH: missing mermaid diagram under ### \`$folder\`"
        exit_code=1
    fi
    is_in_tracked_heading=false
    is_in_mermaid=false
    heading_has_mermaid=false
}

collect_doc() {
    local line
    local is_in_coupling=false
    local from
    local to
    local folder
    current_scope=""
    is_in_tracked_heading=false
    is_in_mermaid=false
    heading_has_mermaid=false
    if [[ ! -f $ARCHITECTURE_DOC_PATH ]]; then
        echo "$ARCHITECTURE_DOC_PATH: file not found"
        exit_code=1
        return
    fi
    while IFS= read -r line || [[ -n $line ]]; do
        if [[ $line =~ $COUPLING_HEADING_REGEX ]]; then
            finish_tracked_heading
            has_coupling_heading=true
            is_in_coupling=true
            continue
        fi
        if [[ $is_in_coupling == true && $line =~ $H2_REGEX ]]; then
            finish_tracked_heading
            is_in_coupling=false
            continue
        fi
        if [[ $is_in_coupling == true && $line =~ $H3_REGEX ]]; then
            finish_tracked_heading
            if [[ $line =~ $TRACKED_HEADING_REGEX ]]; then
                folder="${BASH_REMATCH[1]}"
                start_tracked_heading "$folder" || true
            else
                echo "$ARCHITECTURE_DOC_PATH: invalid coupling heading \`$line\`; expected ### \`src/.../\`"
                exit_code=1
            fi
            continue
        fi
        if [[ $is_in_coupling == true && $line == '```mermaid' ]]; then
            if [[ $is_in_tracked_heading != true ]]; then
                echo "$ARCHITECTURE_DOC_PATH: mermaid diagram is not under a ### \`src/.../\` heading"
                exit_code=1
                continue
            fi
            has_mermaid_diagram=true
            heading_has_mermaid=true
            is_in_mermaid=true
            continue
        fi
        if [[ $is_in_mermaid == true && $line == '```' ]]; then
            is_in_mermaid=false
            continue
        fi
        if [[ $is_in_mermaid != true ]]; then
            continue
        fi
        if [[ $line =~ $MERMAID_SUBGRAPH_REGEX ]]; then
            echo "$ARCHITECTURE_DOC_PATH: nested mermaid subgraphs are not supported; use a \`###\` heading"
            exit_code=1
            continue
        fi
        if [[ $line =~ $MERMAID_EDGE_REGEX ]]; then
            from="${BASH_REMATCH[1]}"
            to="${BASH_REMATCH[3]}"
            add_doc_node "$current_scope" "$from"
            add_doc_node "$current_scope" "$to"
            add_doc_edge "$current_scope" "$from" "$to"
            continue
        fi
        if [[ $line =~ ^[[:space:]]*(graph|flowchart)[[:space:]] ]]; then
            continue
        fi
        if [[ $line =~ ^[[:space:]]*end[[:space:]]*(%%.*)?$ ]]; then
            continue
        fi
        if [[ $line =~ $MERMAID_NODE_REGEX ]]; then
            add_doc_node "$current_scope" "${BASH_REMATCH[1]}"
        fi
    done <"$ARCHITECTURE_DOC_PATH"
    finish_tracked_heading
    if [[ $has_coupling_heading == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing \`## Module coupling\` section"
        exit_code=1
    elif [[ $has_src_heading == false ]]; then
        echo "$ARCHITECTURE_DOC_PATH: missing ### \`src/\` heading in the module coupling section"
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
            echo "$ARCHITECTURE_DOC_PATH: unexpected coupling edge \`$(display_edge "$edge")\` (e.g. in $(code_edge_file "$edge"))"
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
    local folder
    for key in "${scope_module_keys[@]-}"; do
        if ! in_array "$key" "${doc_nodes[@]-}"; then
            scope="${key%%	*}"
            name="${key#*	}"
            folder="$(folder_display "$scope")"
            echo "$ARCHITECTURE_DOC_PATH: missing coupling node \`$name\` (under \`$folder\`)"
            exit_code=1
        fi
    done
    for key in "${doc_nodes[@]-}"; do
        if ! in_array "$key" "${scope_module_keys[@]-}"; then
            scope="${key%%	*}"
            name="${key#*	}"
            folder="$(folder_display "$scope")"
            echo "$ARCHITECTURE_DOC_PATH: extra coupling node \`$name\` (under \`$folder\`)"
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
