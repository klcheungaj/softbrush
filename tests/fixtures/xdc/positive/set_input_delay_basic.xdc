# Test: set_input_delay_basic
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_input_delay -clock clk1 3 [get_ports DIN]

