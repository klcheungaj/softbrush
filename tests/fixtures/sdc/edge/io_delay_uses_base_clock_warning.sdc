create_clock -period 10 -name clk_in [get_ports clk_in]
set_input_delay -clock clk_in -max 3.0 [get_ports data_in]
set_input_delay -clock clk_in -min 0.5 [get_ports data_in]

