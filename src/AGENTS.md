# Source Module Guidance

This directory implements the server in four capability layers:

- `catalog.rs`: static, intentionally incomplete Tcl/SDC/XDC knowledge.
- `syntax.rs`: ANTLR recognition plus tolerant editor-oriented source spans.
- `analysis.rs`: pure diagnostics, semantic classifications, symbols, and
  UTF-8/UTF-16 position conversion.
- `lsp.rs`: protocol conversion and concurrent open-document state.

Keep dependencies flowing in that order. Do not introduce LSP types into the
parser or analyzer, and do not perform I/O from pure analysis functions.

All internal source spans are half-open UTF-8 byte ranges on valid character
boundaries. Convert them to and from zero-based UTF-16 LSP positions only via
`LineIndex`; columns beyond line content must not consume CR/LF bytes. Add
Unicode, CRLF, empty-file, and missing-final-newline regression cases when
position behavior changes.

The tolerant scanner must retain useful commands after malformed or incomplete
input. ANTLR validates the grammar boundary, while the scanner supplies editor
recovery and exact source classifications. Do not make unknown SDC/XDC catalog
commands fatal.

Never edit `generated/*.rs` by hand. Change `grammar/Tcl.g4`, run
`scripts/generate-parser.sh`, and commit the grammar and generated output
together. Generated files are private implementation details.

`DashMap` guards in `lsp.rs` must be dropped before an `.await`. Keep protocol
handlers thin, deterministic, and covered through both analysis tests and the
stdio end-to-end suite.
