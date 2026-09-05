# Test: comments_and_semicolons
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
# Full-line comment
set period_ns 10.0; # End-of-line comment after a semicolon
create_clock -name clk -period $period_ns [get_ports clk]

