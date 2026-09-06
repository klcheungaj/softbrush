# Repository Guidance

## Scope and references

`softbrush_ls` is a tolerant Rust language server for Tcl 8.6 and the SDC/XDC
constraint dialects. Read `docs/coding_practices.md` before changing code. The
ignored `reference/` tree is research material only: committed code, builds,
tests, and documentation must remain functional after it is removed.

## Dependency policy

- Declare Rust crates in the applicable `Cargo.toml` and pin reproducible
  resolution in its `Cargo.lock`. Do not commit Cargo registry sources.
- Add a true non-Cargo dependency as a Git submodule below `third_party/`, with
  its purpose, revision, license, and update procedure documented. Do not add a
  submodule when a maintained Cargo crate is available.
- Keep generated ANTLR output in `src/generated`; it is project build output,
  not a vendored dependency. Regenerate it with `scripts/generate-parser.sh`.
- Review each new dependency for necessity, maintenance, security, and license
  compatibility.

## Architecture

Dependencies flow from `catalog` and `syntax` into pure `analysis`, then into
the stateful `lsp` transport. Keep parser and analysis logic independent of LSP
types. Keep I/O and open-document state in the binary/LSP boundary.

SDC has no universal command standard. Unknown SDC/XDC commands must remain
non-blocking; use hints for likely mistakes and reserve errors for structural
syntax or high-confidence invalid values.

## Build and validation

Use the native GNU target for normal Linux development and tests. Official
Linux artifacts use static musl on x64 and ARM64. Windows artifacts use MSVC
with the static CRT, and macOS artifacts target Apple Silicon; mimalloc is
linked into every executable.

### Development builds

Use GNU builds for normal Linux debugging:

```sh
./scripts/build-linux.sh --gnu
cargo test --locked --all-targets --target x86_64-unknown-linux-gnu
```

The Debian-based `gnu-dev` Docker target includes Rustfmt and Clippy and runs
the test suite by default:

```sh
docker build --target gnu-dev --tag softbrush_ls:dev .
docker run --rm softbrush_ls:dev
```

For an interactive container, bind the repository at `/workspace` and use a
named volume for `/workspace/target`. The image defaults to UID/GID 1000; use
the `UID` and `GID` build arguments when needed.

### Debug token dumps

Debug builds provide an offline report using the same lexer, analyzer, and
semantic encoder as the LSP server:

```sh
cargo run --locked --target x86_64-unknown-linux-gnu -- \
  --dump-tokens tests/fixtures/sdc/edge/token_dump.sdc
```

The command accepts `.tcl`, `.sdc`, and `.xdc` files in argument order. Use
`--dump-tokens -- FILE...` for names beginning with a dash. Reports use the
`softbrush.tokenDump/v1` tab-separated format and contain `lexer`, `word`,
`semantic`, and `diagnostic` rows with UTF-8 byte ranges and UTF-16 positions.
Source diagnostics do not fail the command; usage, file, encoding, and output
errors exit with status 2.

Token dumps exist only with `debug_assertions`. Release builds must reject all
non-transport arguments. Keep `tests/dump_tokens.rs` aligned with the LSP
semantic encoder and preserve its complete byte-coverage and Unicode checks.

### Parser generation

Generated lexer and parser sources are committed under `src/generated`.
Never edit them manually. Change `grammar/Tcl.g4`, then run:

```sh
./scripts/generate-parser.sh
./scripts/generate-parser.sh --check
```

The project-owned generator under `tools/parser-generator` must stay API
compatible with `antlr-rust-runtime`. Normal builds must not require Java or
the generator.

### Validation

```sh
cargo fmt --all --check
cargo test --locked --all-targets --target x86_64-unknown-linux-gnu
cargo clippy --locked --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --target x86_64-unknown-linux-gnu
cargo test --locked --manifest-path tools/parser-generator/Cargo.toml
cargo clippy --locked --manifest-path tools/parser-generator/Cargo.toml --all-targets -- -D warnings
./scripts/generate-parser.sh --check
./scripts/build-linux.sh
./scripts/verify-musl.sh target/x86_64-unknown-linux-musl/release/softbrush_ls
```

When behavior changes, add focused unit/integration coverage and update the
process-level LSP test when the externally observable protocol changes.
`SOFTBRUSH_LS_BIN` selects a prebuilt executable for `tests/lsp_e2e.rs`.

CI tests all five release targets on native GitHub runners and verifies their
linkage policy. Release automation publishes a platform archive and SHA-256
checksum for each target.

## Change discipline

Keep public APIs small and documented. Avoid `unwrap` in production code and
confine any necessary `unsafe` code to `src/ffi/` with a documented safety
contract. Preserve unrelated working-tree changes. Group commits by coherent
purpose and explain generated or dependency-lock changes in the commit.
