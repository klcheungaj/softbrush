# Self-contained language fixtures

These fixtures were selected or written from the golden Tcl 8.6, SDC, and XDC
materials supplied during development. They are intentionally kept in the main
project so builds and tests do not depend on the removable source-material
checkout.

- `tcl/` contains representative Tcl 8.6 standard-library scripts.
- `sdc/` contains valid, invalid, and portability-edge constraint examples.
- `xdc/` contains positive, negative, context-dependent, and unsupported-SDC
  examples.

Fixture comments retain their original expected-result descriptions. Corpus
tests distinguish syntax acceptance from the intentionally conservative lint
surface; a vendor-specific command is not made a hard error merely because it
is absent from the bundled catalog.

## Provenance and licensing

The files under `tcl/` are unmodified representative library scripts from the
Tcl 8.6.18 source distribution. They remain under Tcl's permissive license,
included verbatim as `tcl/license.terms`; their original copyright notices are
retained.

The SDC and XDC files are small, project-authored test inputs derived from the
command forms, interpretation notes, and expected behaviors in the reference
packages supplied for this project. The XDC package was itself based on AMD
Vivado UG835, UG894, and UG903, while the SDC package was based on an
Altera/Intel Quartus Prime Timing Analyzer guide. No reference manual text or
PDF is redistributed here. These project-authored fixtures are covered by the
repository's MIT license.
