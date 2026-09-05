# Test: set_clock_uncertainty
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_clock_uncertainty 0.5 [get_clocks clk]

