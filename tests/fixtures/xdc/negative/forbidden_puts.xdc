# Test: forbidden_puts
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
puts {managed XDC must not use puts}

