# Test: connect_debug_cores_ug835_only
# Expected lexical parse: accept
# Expected managed-XDC command validation: source_dependent
# Expected Vivado execution: context_dependent
connect_debug_cores -master [get_cells debug_bridge_0] -slaves [list [get_cells ila_0] [get_cells ila_1]]

