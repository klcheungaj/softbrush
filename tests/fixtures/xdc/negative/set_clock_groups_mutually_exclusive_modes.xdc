# Test: set_clock_groups_mutually_exclusive_modes
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
set_clock_groups -asynchronous -logically_exclusive -group clk_a -group clk_b

