# Test: create_clock_zero_period
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
create_clock -name clk -period 0 [get_ports clk]

