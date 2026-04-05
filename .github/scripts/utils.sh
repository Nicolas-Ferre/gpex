is_item_in_array() {
    local array=("$@")
    if [ "${#array[@]}" -eq 1 ]; then
        return 1 # Searched item cannot be in empty array
    fi
    ((last_index = ${#array[@]} - 1))
    local searched_item="${array[last_index]}"
    unset "array[last_index]"
    for item in "${array[@]}"; do
        if [[ $item == "$searched_item" ]]; then
            return 0
        fi
    done
    return 1
}
