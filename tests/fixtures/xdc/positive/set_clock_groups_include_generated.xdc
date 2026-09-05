# Test: set_clock_groups_include_generated
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_clock_groups -group [get_clocks -include_generated_clocks src_clk] \
    -group [get_clocks -include_generated_clocks sync_clk] -asynchronous

