# Test: set_false_path_clock_to_clock
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_false_path -from [get_clocks GT0_RXUSRCLK2_OUT] \
    -to [get_clocks DRPCLK_OUT]

