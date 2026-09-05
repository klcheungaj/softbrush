# Test: create_clock_physical
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
create_clock -name bftClk -period 5.000 [get_ports bftClk]

