# Test: set_max_delay_datapath_only
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_max_delay -from [get_pins FF1/C] -to [get_pins FF2/D] -datapath_only 4.0

