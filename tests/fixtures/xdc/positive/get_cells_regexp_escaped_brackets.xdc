# Test: get_cells_regexp_escaped_brackets
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set cells [get_cells -hierarchical -regexp {.*reg\[.*\].*}]

