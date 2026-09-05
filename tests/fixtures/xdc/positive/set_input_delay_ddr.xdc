# Test: set_input_delay_ddr
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
create_clock -name clk_ddr -period 6 [get_ports DDR_CLK_IN]
set_input_delay -clock clk_ddr -max 2.1 [get_ports DDR_IN]
set_input_delay -clock clk_ddr -max 1.9 [get_ports DDR_IN] -clock_fall -add_delay
set_input_delay -clock clk_ddr -min 0.9 [get_ports DDR_IN]
set_input_delay -clock clk_ddr -min 1.1 [get_ports DDR_IN] -clock_fall -add_delay

