# Test: create_debug_core
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
create_debug_core myCore ila
set_property C_DATA_DEPTH 2048 [get_debug_cores myCore]

