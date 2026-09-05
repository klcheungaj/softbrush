# Test: set_power_opt
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_power_opt -include_cells [get_cells -hierarchical *ram*]

