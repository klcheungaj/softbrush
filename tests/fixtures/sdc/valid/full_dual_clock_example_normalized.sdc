# Normalized from guide p. 83.
create_clock -name clockone -period 10.000 [get_ports {clk1}]
create_clock -name clocktwo -period 10.000 [get_ports {clk2}]
create_clock -name clockone_ext -period 10.000
create_clock -name clocktwo_ext -period 10.000
derive_pll_clocks
derive_clock_uncertainty
set_clock_groups \
    -asynchronous \
    -group {clockone} \
    -group {clocktwo {altpll0|altpll_component|auto_generated|pll1|clk[0]}}
set_input_delay -clock {clockone_ext} -max 4 [get_ports {data1}]
set_input_delay -clock {clockone_ext} -min -1 [get_ports {data1}]
set_input_delay -clock {clockone_ext} -max 4 [get_ports {data2}]
set_input_delay -clock {clockone_ext} -min -1 [get_ports {data2}]
set_output_delay -clock {clocktwo_ext} -max 6 [get_ports {dataout}]
set_output_delay -clock {clocktwo_ext} -min -3 [get_ports {dataout}]

