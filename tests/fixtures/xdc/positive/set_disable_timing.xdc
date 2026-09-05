# Test: set_disable_timing
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_disable_timing -from I0 -to O [get_cells LUT_inst]

