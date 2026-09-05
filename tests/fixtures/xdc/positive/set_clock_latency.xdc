# Test: set_clock_latency
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_clock_latency -source 1.2 [get_clocks clk]

