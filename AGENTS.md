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

Use the GNU target for normal development and tests. Official artifacts use
the static musl target with mimalloc.

```sh
cargo fmt --all --check
cargo test --locked --all-targets --target x86_64-unknown-linux-gnu
cargo clippy --locked --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --target x86_64-unknown-linux-gnu
./scripts/generate-parser.sh --check
./scripts/build-musl.sh
./scripts/verify-musl.sh target/x86_64-unknown-linux-musl/release/softbrush_ls
```

When behavior changes, add focused unit/integration coverage and update the
process-level LSP test when the externally observable protocol changes.

## Change discipline

Keep public APIs small and documented. Avoid `unwrap` in production code and
confine any necessary `unsafe` code to `src/ffi/` with a documented safety
contract. Preserve unrelated working-tree changes. Group commits by coherent
purpose and explain generated or dependency-lock changes in the commit.
