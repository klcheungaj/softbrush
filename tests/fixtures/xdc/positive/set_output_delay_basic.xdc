# Test: set_output_delay_basic
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_output_delay 5.0 -clock [get_clocks cpuClk] [get_ports data_out]

