# Test: create_clock_waveform
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
create_clock -name clk -period 10.000 -waveform {2.4 7.4} [get_ports bftClk]

