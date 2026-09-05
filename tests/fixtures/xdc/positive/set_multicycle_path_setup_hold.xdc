# Test: set_multicycle_path_setup_hold
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_multicycle_path 3 -setup -from [get_pins data0_reg/C] \
    -to [get_pins data1_reg/D]
set_multicycle_path 2 -hold -from [get_pins data0_reg/C] \
    -to [get_pins data1_reg/D]

