# Test: forbidden_switch
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
switch -- value {value {set x 1}}

