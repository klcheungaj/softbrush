# Test: physical_query_in_constraint_filter
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: non_recommended
set_false_path -from [get_cells -quiet -hierarchical -filter {REF_NAME =~ FD* && LOC =~ BLI_*}]

