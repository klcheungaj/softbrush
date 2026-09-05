# Test: replicated_objects_filtered_by_name
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: accept_but_incomplete
get_cells -include_replicated_objects -filter {NAME =~ *rx_reg}

