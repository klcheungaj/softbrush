# Test: set_clock_latency_early_without_source
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
set_clock_latency -early 1.0 [get_clocks clk]

