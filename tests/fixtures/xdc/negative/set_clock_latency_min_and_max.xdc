# Test: set_clock_latency_min_and_max
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
set_clock_latency -min -max 1.0 [get_clocks clk]

