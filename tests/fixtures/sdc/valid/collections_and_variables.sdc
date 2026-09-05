set regs1 [get_registers a*]
set regs2 [get_registers b*]
set regs_union [add_to_collection $regs1 $regs2]
set num_items [get_collection_size $regs_union]

