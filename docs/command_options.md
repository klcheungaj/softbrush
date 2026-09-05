# Command option completion

The advisory catalog in `src/catalog/options.rs` supplies option names for
common timing constraints, clock declarations, object queries, and selected
XDC property/Pblock commands. It is independent of linting: an option absent
from completion is not an error, and unknown vendor commands remain non-blocking.
No files under `reference/` are required at build time or runtime.

SDC suggestions use the SDC 1.9 forms documented in AMD's
[Supported SDC Commands comparison (UG903, 2023.2)](https://docs.amd.com/r/2023.2-English/ug903-vivado-using-constraints/Supported-SDC-Commands).
XDC suggestions use the AMD column, supplemented by the more detailed
[Vivado Tcl command reference (UG835)](https://docs.amd.com/r/2024.1-English/ug835-vivado-tcl-commands/get_ports).
The catalog is intentionally incomplete; SDC tool extensions do not have a
single universal specification.

The following command pages clarify the options used here:

- [set_input_delay (UG835, 2023.1)](https://docs.amd.com/r/2023.1-English/ug835-vivado-tcl-commands/set_input_delay): `-clock_fall` is the falling-edge switch. `-clock_fail` is not a documented spelling. Intel's [Timing Analyzer SDC reference](https://cdrdv2-public.intel.com/655074/mnl_sdctmq.pdf) uses the same spelling.
- [create_generated_clock (UG835, 2024.2)](https://docs.amd.com/r/2024.2-English/ug835-vivado-tcl-commands/create_generated_clock): includes `-invert`, which is omitted from the older comparison table's AMD column.
- [get_pins (UG835, 2021.2)](https://docs.amd.com/r/2021.2-English/ug835-vivado-tcl-commands/get_pins): includes `-include_replicated_objects`.
- [set_property (UG835)](https://docs.amd.com/r/en-US/ug835-vivado-tcl-commands/set_property): includes `-dict`.
- [resize_pblock (UG835)](https://docs.amd.com/r/en-US/ug835-vivado-tcl-commands/resize_pblock) and [add_cells_to_pblock (UG835)](https://docs.amd.com/r/en-US/ug835-vivado-tcl-commands/add_cells_to_pblock): Pblock options.

For example, `set_input_delay -` offers the common delay switches in both
languages, `-level_sensitive` in SDC, and `-reference_pin`, `-quiet`, and
`-verbose` in XDC. Options are filtered by the text before the cursor; accepting
one replaces the entire option word, including its existing hyphen and suffix.
Nested substitutions use the inner command's catalog. Quoted/braced literals,
comments, and signed numbers do not create option-completion contexts.

To extend the catalog, verify the command's syntax in the relevant vendor
reference, preserve dialect distinctions, add a focused completion regression,
and record any additional source here. The catalog contains manually maintained
option names, not generated parser output or copied vendor documentation.
