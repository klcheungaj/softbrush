# Test: startgroup_endgroup
# Expected lexical parse: accept
# Expected managed-XDC command validation: accept
# Expected Vivado execution: context_dependent
startgroup
create_pblock pblock_wbArbEngine
add_cells_to_pblock pblock_wbArbEngine [get_cells wbArbEngine] -clear_locs
endgroup

