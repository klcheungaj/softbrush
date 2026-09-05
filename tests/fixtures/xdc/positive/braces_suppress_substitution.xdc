# Test: braces_suppress_substitution
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set literal_text {The value is [expr {1 + 1}] and $period_ns}

