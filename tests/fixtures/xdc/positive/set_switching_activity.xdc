# Test: set_switching_activity
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_switching_activity -toggle_rate 12.5 -static_probability 0.5 [get_nets data_net]

