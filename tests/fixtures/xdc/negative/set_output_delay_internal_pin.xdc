# Test: set_output_delay_internal_pin
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: reject
set_output_delay 2.0 -clock [get_clocks clk] [get_pins u0/data_reg/Q]

