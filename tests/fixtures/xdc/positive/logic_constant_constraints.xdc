# Test: logic_constant_constraints
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_logic_zero [get_ports tie_low]
set_logic_one [get_ports tie_high]
set_logic_dc [get_ports dont_care]
set_logic_unconnected [get_ports unused]

