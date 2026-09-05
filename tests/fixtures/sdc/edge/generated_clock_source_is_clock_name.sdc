create_clock -period 10 -name base_clk [get_ports clk]
# Source guide says -source must be a design netlist node, not a previously defined clock name.
create_generated_clock -name gen_clk -divide_by 2 -source base_clk [get_pins reg|q]

