# Test: filter_boolean_and_string
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set flops [get_cells -hierarchical -filter {IS_PRIMITIVE && REF_NAME =~ FD*}]

