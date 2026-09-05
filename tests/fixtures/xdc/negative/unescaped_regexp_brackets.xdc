# Test: unescaped_regexp_brackets
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: warning_no_match
get_cells -hierarchical -regexp {.*reg[.*].*}

