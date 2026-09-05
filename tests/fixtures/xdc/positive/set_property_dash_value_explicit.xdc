# Test: set_property_dash_value_explicit
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set_property -name USER_VALUE -value -leading_dash -objects [get_cells u0]

