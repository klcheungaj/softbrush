# Test: set_max_delay_datapath_only_without_from
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
set_max_delay 4.0 -datapath_only -to [get_pins FF2/D]

