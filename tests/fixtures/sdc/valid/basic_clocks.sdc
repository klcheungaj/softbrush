# Basic base clocks.
create_clock -period 20.00 -name adc_clk [get_ports adc_clk]
create_clock -period 8.00 -name sys_clk [get_ports sys_clk]
derive_pll_clocks
derive_clock_uncertainty

