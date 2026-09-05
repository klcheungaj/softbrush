# Test: create_generated_clock_add_without_name
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
create_generated_clock -add -source [get_ports clk] [get_pins mux/Q]

