# Test: get_cells_of_objects_with_pattern
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
get_cells -of_objects [get_nets data] u*

