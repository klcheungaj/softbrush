# Test: add_cells_to_pblock_top_with_cells
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
add_cells_to_pblock -top [get_pblocks p0] [get_cells u0]

