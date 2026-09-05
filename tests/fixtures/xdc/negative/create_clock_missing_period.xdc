# Test: create_clock_missing_period
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
create_clock -name clk [get_ports clk]

