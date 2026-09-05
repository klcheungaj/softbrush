# Test: create_generated_clock_divide
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
create_generated_clock -divide_by 2 -source \
    [get_pins clkgen/cpuClk] [get_nets fftEngine/CLK]

