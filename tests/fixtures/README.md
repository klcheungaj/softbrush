# Self-contained language fixtures

These fixtures were selected or written from the golden Tcl 8.6, SDC, and XDC
materials supplied during development. They are intentionally kept in the main
project so builds and tests do not depend on the removable source-material
checkout.

- `tcl/` contains representative Tcl 8.6 standard-library scripts.
- `sdc/` contains valid, invalid, and portability-edge constraint examples.
- `xdc/` contains positive, negative, context-dependent, and unsupported-SDC
  examples.

`sdc/edge/token_dump.sdc` is a project-authored regression for signed numeric
tokens, Unicode clock names, unresolved and forward references, and vendor
commands that must not declare clocks. It is used by the dump and LSP tests.

Fixture comments retain their original expected-result descriptions. Corpus
tests distinguish syntax acceptance from the intentionally conservative lint
surface; a vendor-specific command is not made a hard error merely because it
is absent from the bundled catalog.

## Provenance and licensing

The files under `tcl/` are unmodified representative library scripts from the
Tcl 8.6.18 source distribution. They remain under Tcl's permissive license,
included verbatim as `tcl/license.terms`; their original copyright notices are
retained.

The Tcl corpus contains 14 scripts. Paths under `tcl/` preserve their paths
relative to the upstream Tcl 8.6.18 `library/` directory:

| Fixture paths | Representative library code |
| --- | --- |
| `init.tcl`, `package.tcl`, `safe.tcl`, `clock.tcl` | Initialization, package management, safe interpreters, and clock handling |
| `auto.tcl` | Autoloading and index generation |
| `history.tcl` | History commands and namespace ensembles |
| `parray.tcl` | Array formatting and iteration |
| `tm.tcl` | Tcl module discovery and namespace paths |
| `word.tcl` | Word boundaries and regular expressions |
| `http/http.tcl` | HTTP callbacks, state, and channel handling |
| `msgcat/msgcat.tcl` | Message catalogs, locale handling, and dictionaries |
| `opt/optparse.tcl` | Option parsing and argument descriptions |
| `platform/platform.tcl`, `platform/shell.tcl` | Platform detection and shell configuration |

These are parser/analyzer inputs, not runtime dependencies; the tests do not
execute them or require the packages, network access, or platform facilities
they reference. The corpus test recursively discovers `.tcl` files, checks
both parser implementations for structural errors, and rejects error-level
analysis diagnostics. The license file is retained for redistribution and is
not parsed as Tcl.

To refresh a script, copy the corresponding upstream `library/` file verbatim,
retain the distribution's license and copyright notices, update this provenance
record, and run the GNU `reference_corpus` integration test. The local source
checkout used for these copies was `reference/tcl8.6.18`; the copied fixtures
remain usable after that checkout is removed.

The SDC and XDC files are small, project-authored test inputs derived from the
command forms, interpretation notes, and expected behaviors in the reference
packages supplied for this project. The XDC package was itself based on AMD
Vivado UG835, UG894, and UG903, while the SDC package was based on an
Altera/Intel Quartus Prime Timing Analyzer guide. No reference manual text or
PDF is redistributed here. These project-authored fixtures are covered by the
repository's license (see the root `LICENSE`).
