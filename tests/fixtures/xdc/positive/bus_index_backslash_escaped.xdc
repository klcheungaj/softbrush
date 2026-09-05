# Test: bus_index_backslash_escaped
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set pins [get_pins transformLoop\[0\].ct/xOutReg_reg/CARRYOUT\[*\]]

