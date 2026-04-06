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
