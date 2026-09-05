# Test: filter_property_on_right_operand
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
get_cells -hierarchical -filter {FDRE == REF_NAME}

