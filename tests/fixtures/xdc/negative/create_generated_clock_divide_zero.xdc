# Test: create_generated_clock_divide_zero
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
create_generated_clock -divide_by 0 -source [get_ports clk] [get_pins div_reg/Q]

