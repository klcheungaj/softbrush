# softbrush_ls

`softbrush_ls` is a Rust language server for Tcl 8.6 scripts and the Tcl-based
SDC/XDC constraint dialects. It uses an ANTLR4 grammar and the pure-Rust
`antlr-rust-runtime` generated recognizer.

## Features

- `.tcl`, `.sdc`, and `.xdc` documents over standard LSP stdio
- semantic highlighting for commands, Tcl keywords, variables, strings,
  signed numbers, SDC/XDC options, namespaces, comments, and clock references
  that resolve to earlier clock declarations
- delimiter/substitution syntax errors and basic Tcl command linting
- intentionally tolerant SDC/XDC catalog checks, reported as hints
- high-confidence checks for common clock, delay, and multicycle mistakes
- document and workspace symbols for procedures, namespaces, variables,
  clocks, Pblocks, macros, and debug cores
- command completion and hover help
- UTF-16-correct LSP positions

## Build targets

For inspecting recognized tokens without an editor, see [debug token dumps](#debug-token-dumps).

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

## Debug token dumps

Debug builds provide an offline inspection command, modeled on lapligence's
`--dump-tokens`. It runs the same analyzer and semantic-token encoder used by
the language server:

```sh
cargo run --locked --target x86_64-unknown-linux-gnu -- \
  --dump-tokens tests/fixtures/sdc/edge/token_dump.sdc

# Multiple files, including shell globs, are accepted in argument order.
target/x86_64-unknown-linux-gnu/debug/softbrush_ls --dump-tokens constraints/*.sdc
target/x86_64-unknown-linux-gnu/debug/softbrush_ls --help
```

Pass `.tcl`, `.sdc`, or `.xdc` files (extensions are case insensitive). Use
`--dump-tokens -- FILE...` for filenames starting with a dash. Directory
arguments are not supported. Reports use tab-separated rows under a
`softbrush.tokenDump/v1` header, with these layers:

- `lexer`: all ANTLR tokens, including whitespace, delimiters, and EOF.
- `word`: Tcl command names and arguments, with substitution nesting depth.
- `semantic`: the actual LSP token types and declaration modifiers, interleaved
  with `unclassified` ranges so missing highlighting is visible.
- `diagnostic`: source errors and lint messages; the summary also includes the
  ANTLR parser error count.

Each row contains a half-open UTF-8 byte range, zero-based UTF-16 start/end
positions, kind, metadata, and Rust debug-escaped source text. Tabs and newlines
inside source text are escaped. EOF has an empty range. Semantic and lexer rows
are ordered by source position; word rows group arguments by command, so nested
commands can overlap their containing word. The output has no timestamps or
colors and can be saved or diffed. Source errors remain report data (exit 0);
usage, file, encoding, or output errors go to stderr and exit 2.

The command is compiled only with `debug_assertions` (normal `cargo build` and
`cargo run`). GNU and musl release builds reject command-line arguments with
exit 2; no dump implementation is included. Starting with no arguments still
serves LSP over stdio without printing a token report.

SDC/XDC switches use `keyword`, brace delimiters use `operator`, signed values
use `number`, and resolved
literal clock names use `variable`. Clock lookup is local to the current
document, case sensitive, and uses preceding `create_clock` or
`create_generated_clock` declarations. A missing literal name following
`-clock`, including the contents of a quoted or braced name, receives no
semantic token. Braces remain separately classified. This gives options and
braces distinct semantic categories from signal names and string contents.
The editor's theme and lexical grammar still determine its displayed color.
Default names can be inferred from a single literal target or a simple
`[get_ports name]` / `[get_pins name]` query. Wildcard queries, tool-derived
clocks such as `derive_pll_clocks`, external files, and Tcl evaluation (including
procedure bodies and dynamically computed names) are not resolved.

Run the CLI regression suite with `cargo test --locked --test dump_tokens`.
The LSP end-to-end suite compares the dump against tokens served over JSON-RPC
and checks that deleting a clock definition removes its reference highlighting.

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

The self-contained integration corpus under `tests/fixtures` contains Tcl 8.6
library samples, 177 supplied SDC/XDC examples, and a token-dump regression input.
Every fixture is analyzed at the supported layer. Unknown constraint commands
remain legal because real SDC environments are vendor-extensible.

The test suite covers:

- ANTLR and tolerant-parser acceptance across the valid Tcl/SDC/XDC corpus
- syntax recovery (including excessive nesting), continuation, command, and
  constraint-value failures
- every semantic-token and symbol category, including SDC/XDC switches, signed
  numeric values, and source-ordered clock references
- exact UTF-16 semantic/diagnostic ranges across Unicode, CRLF, and multiline
  source, plus dialect selection
- a real `softbrush_ls` child process over framed JSON-RPC, including all
  advertised capabilities, diagnostics after open/change/close, semantic
  tokens, document/workspace symbols, hover, completion, shutdown, and exit

Run only the process-level suite with `cargo test --test lsp_e2e`. Set
`SOFTBRUSH_LS_BIN` to exercise a prebuilt executable such as the musl release.
