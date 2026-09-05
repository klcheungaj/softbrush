# Token audit: options, signed numbers, and clock lookup
create_clock -name clk_😀 -period 10
set_input_delay -clock_fall -clock clk_😀 -max -1.25 [get_ports din]
set_output_delay -clock {missing} -min +.5 [get_ports dout]
vendor_command -name impostor
set_input_delay -clock impostor -max -2e-3 din
set_input_delay -clock future -max 1 din
create_generated_clock -name future -source pin -master_clock clk_😀 out
