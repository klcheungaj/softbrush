# Test: set_data_check
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_data_check -from [get_pins source_reg/Q] -to [get_pins dest_reg/D] 2.0

