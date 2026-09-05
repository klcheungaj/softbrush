# Test Guidance

Tests cover observable Tcl/SDC/XDC behavior and the stdio JSON-RPC boundary.
Keep them deterministic, hermetic, and independent of the ignored `reference/`
tree.

Place compact behavior regressions in `analysis_features.rs`, corpus parsing in
`reference_corpus.rs`, and advertised LSP behavior in `lsp_e2e.rs`. Fixtures
belong under `fixtures/<dialect>/<category>/`; document their provenance and
license in `fixtures/README.md`. Prefer small representative inputs over copied
manuals or large vendor test suites.

Unknown SDC/XDC commands are valid extension points. Test them as advisory
hints, alongside separate cases for definite syntax errors and high-confidence
constraint mistakes. For protocol tests, use bounded waits, complete the LSP
shutdown/exit handshake, and ensure failed tests terminate child processes.

`SOFTBRUSH_LS_BIN` may select a prebuilt binary, including the musl release;
without it, Cargo's test-built `softbrush_ls` is used.
