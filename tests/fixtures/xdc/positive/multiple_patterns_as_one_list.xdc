# Test: multiple_patterns_as_one_list
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set cells [get_cells -hierarchical {cpu* fft*}]

