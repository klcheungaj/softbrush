# Test: create_clock_used_for_generated_clock
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: accept_but_incorrect_timing_model
create_clock -name divided_clk -period 20 [get_pins divider_reg/Q]

