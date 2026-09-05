# Test: move_pblock_ug835_only
# Expected lexical parse: accept
# Expected managed-XDC command validation: source_dependent
# Expected Vivado execution: context_dependent
move_pblock -from CLOCKREGION_X0Y0:CLOCKREGION_X1Y1 -to CLOCKREGION_X2Y2:CLOCKREGION_X3Y3 -locs trim [get_pblocks p0]

