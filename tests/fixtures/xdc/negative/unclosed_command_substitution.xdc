# Test: unclosed_command_substitution
# Expected lexical parse: reject
# Expected managed-XDC command validation: not_reached
# Expected Vivado execution: reject
set value [expr {1 + 2}

