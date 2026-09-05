# Test: set_bus_skew
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_bus_skew -from [get_pins src_reg[*]/Q] -to [get_pins dst_reg[*]/D] 0.500

