//! Parsing, analysis, command catalogs, and LSP integration for `softbrush_ls`.

#![warn(missing_docs)]

/// Dialect-aware diagnostics, semantic classifications, and symbols.
pub mod analysis;
/// Tcl, SDC, and XDC command catalogs.
pub mod catalog;
#[allow(missing_docs)]
mod generated;
/// Language Server Protocol integration.
pub mod lsp;
/// Tolerant Tcl source parsing and syntax models.
pub mod syntax;
