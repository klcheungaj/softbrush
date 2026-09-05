# Test: create_clock_odd_waveform_edges
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
create_clock -name clk -period 10 -waveform {0 2 5} [get_ports clk]

