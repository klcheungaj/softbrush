# Test: get_cells_filter_name
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set cells [get_cells -hierarchical -filter {NAME =~ B/b* && !IS_LOC_FIXED}]

