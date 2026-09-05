# Test: get_pins_relationship
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
set pins [get_pins -of_objects [get_nets -hierarchical *]]

