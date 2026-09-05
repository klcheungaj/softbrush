# softbrush_ls

`softbrush_ls` is a Rust language server for Tcl 8.6 scripts and the Tcl-based
SDC/XDC constraint dialects. It uses an ANTLR4 grammar and the pure-Rust
`antlr-rust-runtime` generated recognizer.

## Features

- `.tcl`, `.sdc`, and `.xdc` documents over standard LSP stdio
- semantic highlighting for commands, Tcl keywords, variables, strings,
  numbers, options, namespaces, and comments
- delimiter/substitution syntax errors and basic Tcl command linting
- intentionally tolerant SDC/XDC catalog checks, reported as hints
- high-confidence checks for common clock, delay, and multicycle mistakes
- document and workspace symbols for procedures, namespaces, variables,
  clocks, Pblocks, macros, and debug cores
- command completion and hover help
- UTF-16-correct LSP positions

## Build targets

The executable uses mimalloc as its global allocator with both supported Linux
targets. GNU is the recommended development and test target, particularly in
containers where glibc tooling is expected. Official release artifacts use
musl.

### GNU development builds

The GNU target is normally installed with Rust on a glibc-based Linux host:

```sh
cargo build --locked --target x86_64-unknown-linux-gnu
cargo test --locked --all-targets --target x86_64-unknown-linux-gnu
./target/x86_64-unknown-linux-gnu/debug/softbrush_ls
```

The `gnu-dev` Docker target is Debian-based and includes Rustfmt and Clippy. Its
default command runs the complete test suite using GNU:

```sh
docker build --target gnu-dev --tag softbrush_ls:dev .
docker run --rm softbrush_ls:dev
```

For an interactive development container with the repository mounted:

```sh
docker run --rm --interactive --tty \
  --mount type=bind,source="$(pwd)",target=/workspace \
  --mount type=volume,source=softbrush-target,target=/workspace/target \
  softbrush_ls:dev bash
```

The development image uses UID/GID 1000 by default. If necessary, pass `UID`
and `GID` as Docker build arguments when building `gnu-dev`.

A GNU release executable can also be exported with Docker BuildKit:

```sh
docker build --target gnu-artifact --output type=local,dest=dist-gnu .
./dist-gnu/softbrush_ls
```

### MUSL release builds

For a host build, install the Rust target and a musl C compiler, then use the
checked-in wrapper:

```sh
rustup target add x86_64-unknown-linux-musl
# Debian/Ubuntu: sudo apt-get install musl-tools
./scripts/build-musl.sh
./target/x86_64-unknown-linux-musl/release/softbrush_ls
```

The wrapper automatically finds the usual musl compiler names and builds with
`--locked`. `MUSL_CC` is only needed when the compiler has a nonstandard name.

Official artifacts are built in the Alpine `musl-builder` stage. Export the
static executable or build the runnable release image with:

```sh
docker build --target musl-artifact --output type=local,dest=dist-musl .
./dist-musl/softbrush_ls

docker build --tag softbrush_ls:musl .
docker run --rm --interactive softbrush_ls:musl
```

The final Alpine image runs `softbrush_ls` as an unprivileged user.
Publishing a GitHub release builds this same musl target and uploads a tarball
and SHA-256 checksum; GNU binaries remain development artifacts.

Configure an editor language client to launch `softbrush_ls` over stdio for the
language IDs `tcl`, `sdc`, and `xdc` and the matching file extensions.

## Parser generation

Generated Rust lexer/parser sources are committed under `src/generated`, so a
normal build does not require Java or a generator installation. The project-owned
helper under `tools/parser-generator` uses the matching Rust generator crate:

```sh
./scripts/generate-parser.sh
./scripts/generate-parser.sh --check
```

The first command updates the generated source. The non-mutating `--check`
form is used by CI to detect grammar/generated-source drift. The generated
source and runtime dependency must use compatible codegen API revisions.

Cargo dependencies for both the server and parser generator are declared in
their `Cargo.toml` files and pinned by their `Cargo.lock` files. Cargo downloads
them from the registry during builds; crate source trees are not committed to
this repository. Any future non-Cargo dependency must be added as a Git
submodule under `third_party/`.

## Validation

```sh
cargo fmt --all --check
cargo test --locked --all-targets --target x86_64-unknown-linux-gnu
cargo clippy --locked --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps --target x86_64-unknown-linux-gnu
cargo test --locked --manifest-path tools/parser-generator/Cargo.toml
cargo clippy --locked --manifest-path tools/parser-generator/Cargo.toml --all-targets -- -D warnings
./scripts/build-musl.sh
./scripts/verify-musl.sh target/x86_64-unknown-linux-musl/release/softbrush_ls
```

CI checks formatting, tests and lints the GNU target used during development,
and audits both Cargo lockfiles. Separately, it builds the musl release
executable, verifies that the ELF file is static and uses mimalloc, and runs the
JSON-RPC end-to-end suite against that exact executable. Container CI likewise
tests `gnu-dev` and smoke-tests the Alpine musl image.

The generator audit may report unmaintained UNIC crates pulled transitively by
`antlr-rust-codegen` through RustPython. They are isolated to parser generation,
not linked into `softbrush_ls`; CI still fails on vulnerabilities and
unsoundness, and Dependabot tracks upstream dependency updates.

The self-contained integration corpus under `tests/fixtures` contains 182
fixtures, including Tcl 8.6 library samples and 177 supplied SDC/XDC examples.
Every fixture is analyzed at the supported layer. Unknown constraint commands
remain legal because real SDC environments are vendor-extensible.

The test suite covers:

- ANTLR and tolerant-parser acceptance across the valid Tcl/SDC/XDC corpus
- syntax recovery (including excessive nesting), continuation, command, and
  constraint-value failures
- every semantic-token and symbol category
- exact UTF-16 semantic/diagnostic ranges across Unicode, CRLF, and multiline
  source, plus dialect selection
- a real `softbrush_ls` child process over framed JSON-RPC, including all
  advertised capabilities, diagnostics after open/change/close, semantic
  tokens, document/workspace symbols, hover, completion, shutdown, and exit

Run only the process-level suite with `cargo test --test lsp_e2e`. Set
`SOFTBRUSH_LS_BIN` to exercise a prebuilt executable such as the musl release.
