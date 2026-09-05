# Test: set_property_package_pin
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_property PACKAGE_PIN A1 [get_ports data_in]
set_property IOSTANDARD LVCMOS18 [get_ports data_in]

