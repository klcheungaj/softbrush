# Test: filter_existing_collection
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set all_cells [get_cells -hierarchical *]
set primitive_cells [filter $all_cells {IS_PRIMITIVE}]

