# Test: clock_referenced_before_definition
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: reject_or_ignore_constraint
set_input_delay -clock late_clock 2.0 [get_ports DIN]
create_clock -name late_clock -period 10 [get_ports CLK]

