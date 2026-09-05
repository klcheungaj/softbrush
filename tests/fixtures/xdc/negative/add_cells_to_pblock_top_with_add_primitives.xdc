# Test: add_cells_to_pblock_top_with_add_primitives
# Expected lexical parse: accept
# Expected managed-XDC command validation: reject
# Expected Vivado execution: reject
add_cells_to_pblock -top -add_primitives [get_pblocks p0]

