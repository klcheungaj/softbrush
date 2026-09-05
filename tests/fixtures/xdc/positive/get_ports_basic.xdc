# Test: get_ports_basic
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set ports [get_ports {clk reset_n data[*]}]

