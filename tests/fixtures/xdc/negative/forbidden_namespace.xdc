# Test: forbidden_namespace
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
namespace eval helper {variable x 1}

