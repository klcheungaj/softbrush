# Test: line_continuation
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
create_clock -name clk \
    -period 10.000 \
    -waveform {0.000 5.000} \
    [get_ports clk]

