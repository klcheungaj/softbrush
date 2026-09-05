# Test: forbidden_for
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
for {set i 0} {$i < 4} {incr i} {set x $i}

