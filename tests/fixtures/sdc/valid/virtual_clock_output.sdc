create_clock -period 5 [get_ports system_clk]
create_clock -period 10 -name virt_clk
set_output_delay -clock virt_clk -max 1.5 [get_ports dataout]
set_output_delay -clock virt_clk -min 0.0 [get_ports dataout]

