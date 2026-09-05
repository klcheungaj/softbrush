# Test: get_cells_hierarchical
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set cells [get_cells -hierarchical b*]

