# Test: set_property_dash_value_positional
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
set_property USER_VALUE -leading_dash [get_cells u0]

