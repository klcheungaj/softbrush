# Test: create_generated_clock_missing_object
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
create_generated_clock -divide_by 2 -source [get_ports clk]

