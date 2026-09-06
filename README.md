# softbrush_ls

`softbrush_ls` is a tolerant language server for Tcl 8.6 and the Tcl-based
SDC/XDC constraint dialects.

## Features

- Syntax diagnostics and Tcl command linting
- Semantic highlighting with Unicode-correct positions
- SDC/XDC validation that allows vendor extensions
- Document and workspace symbols
- Context-aware completion and hover help
- Go-to-definition for document-local clock references

## Supported platforms

| Platform | Architecture | Release linkage |
| --- | --- | --- |
| Linux | x64, ARM64 | Fully static musl executable |
| Windows | x64, ARM64 | Static MSVC runtime |
| macOS | ARM64 | Apple system libraries remain dynamic |

Release archives and SHA-256 checksums are published for each platform.

## Build

Linux builds default to a static musl release for the host architecture:

```sh
rustup target add "$(uname -m)-unknown-linux-musl"
./scripts/build-linux.sh
```

Use `--arch x86_64` or `--arch aarch64` to select an architecture. Set
`MUSL_CC` if the musl compiler has a nonstandard name. An Alpine artifact can
also be exported with Docker:

```sh
docker build --target musl-artifact --output type=local,dest=dist .
```

On Windows, build from an MSVC developer shell:

```powershell
cargo build --locked --release --target x86_64-pc-windows-msvc
cargo build --locked --release --target aarch64-pc-windows-msvc
```

On Apple Silicon:

```sh
cargo build --locked --release --target aarch64-apple-darwin
```

## Usage

Configure your editor's language client to run `softbrush_ls --stdio` for
`.tcl`, `.sdc`, and `.xdc` files. Starting the executable without arguments
also serves LSP over stdio.

## License

Licensed under the [GNU General Public License v2.0](LICENSE).
