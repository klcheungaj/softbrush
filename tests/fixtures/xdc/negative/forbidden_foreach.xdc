# Test: forbidden_foreach
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
foreach p [get_ports *] {set_property IOSTANDARD LVCMOS18 $p}

