# shellcheck disable=SC2034 # used by scripts sourcing this file
VISIBILITY_KINDS=("pub" "pub(crate)" "pub(super)" "pub(<other>)" "private")

in_array() {
    local searched_item="$1"
    shift
    local item
    for item in "$@"; do
        if [[ $item == "$searched_item" ]]; then
            return 0
        fi
    done
    return 1
}

compute_visibility_rank() {
    local item_line="$1"
    if [[ $item_line =~ ^pub[[:space:]] ]]; then
        echo 0
    elif [[ $item_line =~ ^pub\(crate\)[[:space:]] ]]; then
        echo 1
    elif [[ $item_line =~ ^pub\(super\)[[:space:]] ]]; then
        echo 2
    elif [[ $item_line =~ ^pub\([^\)]+\)[[:space:]] ]]; then
        echo 3
    else
        echo 4
    fi
}
