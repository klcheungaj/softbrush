//! LSP request handling and conversion from internal byte spans to LSP data.

/// Offline token inspection using the same analysis and token encoder as LSP.
#[cfg(debug_assertions)]
pub mod debug_dump;

use std::collections::BTreeSet;
use std::ops::Range as ByteRange;

use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    CompletionTextEdit, DiagnosticSeverity, DidChangeTextDocumentParams,
    DidCloseTextDocumentParams, DidOpenTextDocumentParams, DocumentSymbol, DocumentSymbolParams,
    DocumentSymbolResponse, Documentation, GotoDefinitionParams, GotoDefinitionResponse, Hover,
    HoverContents, HoverParams, HoverProviderCapability, InitializeParams, InitializeResult,
    InitializedParams, Location, MarkupContent, MarkupKind, MessageType, NumberOrString, OneOf,
    Position, PositionEncodingKind, Range, SemanticToken, SemanticTokenModifier, SemanticTokenType,
    SemanticTokens, SemanticTokensFullOptions, SemanticTokensLegend, SemanticTokensOptions,
    SemanticTokensParams, SemanticTokensResult, SemanticTokensServerCapabilities,
    ServerCapabilities, ServerInfo, SymbolInformation, SymbolKind, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextEdit, Url, WorkspaceSymbolParams,
};
use tower_lsp::{Client, LanguageServer};

use crate::analysis::{
    Analysis, LineIndex, SemanticKind, Severity, SymbolKind as InternalSymbolKind, analyze,
};
use crate::catalog::{self, Dialect};

const TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::COMMENT,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::NAMESPACE,
];

const TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[SemanticTokenModifier::DECLARATION];

#[derive(Clone, Debug)]
struct Document {
    text: String,
    dialect: Dialect,
    index: LineIndex,
    analysis: Analysis,
}

impl Document {
    fn new(uri: &Url, text: String) -> Self {
        let dialect = Dialect::from_uri_path(uri.path());
        let index = LineIndex::new(&text);
        let analysis = analyze(&text, dialect);
        Self {
            text,
            dialect,
            index,
            analysis,
        }
    }
}

/// Stateful LSP backend for open Tcl, SDC, and XDC documents.
pub struct Backend {
    client: Client,
    documents: DashMap<Url, Document>,
}

impl Backend {
    /// Creates a backend that publishes diagnostics through `client`.
    #[must_use]
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    async fn update_document(&self, uri: Url, text: String) {
        let document = Document::new(&uri, text);
        let diagnostics = to_lsp_diagnostics(&document);
        self.documents.insert(uri.clone(), document);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                position_encoding: Some(PositionEncodingKind::UTF16),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: SemanticTokensLegend {
                                token_types: TOKEN_TYPES.to_vec(),
                                token_modifiers: TOKEN_MODIFIERS.to_vec(),
                            },
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            range: None,
                            ..SemanticTokensOptions::default()
                        },
                    ),
                ),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["-".to_owned()]),
                    ..CompletionOptions::default()
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "softbrush_ls".to_owned(),
                version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(
                MessageType::INFO,
                "softbrush_ls initialized for Tcl, SDC, and XDC",
            )
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update_document(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.into_iter().last() {
            self.update_document(params.text_document.uri, change.text)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, Vec::new(), None)
            .await;
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let Some(document) = self.documents.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let data = encode_semantic_tokens(&document);
        Ok(Some(SemanticTokensResult::Tokens(SemanticTokens {
            result_id: None,
            data,
        })))
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let Some(document) = self.documents.get(&params.text_document.uri) else {
            return Ok(None);
        };
        let symbols = document
            .analysis
            .symbols
            .iter()
            .map(|symbol| DocumentSymbol {
                name: symbol.name.clone(),
                detail: Some(symbol.detail.to_owned()),
                kind: match symbol.kind {
                    InternalSymbolKind::Function => SymbolKind::FUNCTION,
                    InternalSymbolKind::Variable => SymbolKind::VARIABLE,
                    InternalSymbolKind::Namespace => SymbolKind::NAMESPACE,
                    InternalSymbolKind::Clock | InternalSymbolKind::Object => SymbolKind::OBJECT,
                },
                tags: None,
                #[allow(deprecated)]
                deprecated: None,
                range: to_range(&document.index, symbol.span.clone()),
                selection_range: to_range(&document.index, symbol.selection_span.clone()),
                children: None,
            })
            .collect();
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    async fn symbol(
        &self,
        params: WorkspaceSymbolParams,
    ) -> Result<Option<Vec<SymbolInformation>>> {
        let query = params.query.to_ascii_lowercase();
        let mut symbols = Vec::new();
        for document in &self.documents {
            symbols.extend(
                document
                    .analysis
                    .symbols
                    .iter()
                    .filter(|symbol| symbol.name.to_ascii_lowercase().contains(&query))
                    .map(|symbol| SymbolInformation {
                        name: symbol.name.clone(),
                        kind: match symbol.kind {
                            InternalSymbolKind::Function => SymbolKind::FUNCTION,
                            InternalSymbolKind::Variable => SymbolKind::VARIABLE,
                            InternalSymbolKind::Namespace => SymbolKind::NAMESPACE,
                            InternalSymbolKind::Clock | InternalSymbolKind::Object => {
                                SymbolKind::OBJECT
                            }
                        },
                        tags: None,
                        #[allow(deprecated)]
                        deprecated: None,
                        location: Location::new(
                            document.key().clone(),
                            to_range(&document.index, symbol.selection_span.clone()),
                        ),
                        container_name: Some(symbol.detail.to_owned()),
                    }),
            );
        }
        Ok(Some(symbols))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let Some(document) = self.documents.get(uri) else {
            return Ok(None);
        };
        let position = params.text_document_position_params.position;
        let offset = document.index.offset(position.line, position.character);
        let Some(command) = document.analysis.syntax.commands.iter().find(|command| {
            command
                .words
                .first()
                .is_some_and(|word| word.span.start <= offset && offset <= word.span.end)
        }) else {
            return Ok(None);
        };
        let Some(name) = command.name() else {
            return Ok(None);
        };
        let description = catalog::command_summary(name).unwrap_or(
            "Tcl-style command. Its accepted arguments may depend on the active tool or dialect.",
        );
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("`{name}`\n\n{description}"),
            }),
            range: command
                .words
                .first()
                .map(|word| to_range(&document.index, word.span.clone())),
        }))
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let Some(document) = self.documents.get(uri) else {
            return Ok(None);
        };
        let position = params.text_document_position_params.position;
        let offset = document.index.offset(position.line, position.character);
        Ok(document.analysis.definition_at(offset).map(|span| {
            GotoDefinitionResponse::Scalar(Location::new(
                uri.clone(),
                to_range(&document.index, span),
            ))
        }))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        if let Some(document) = self.documents.get(uri) {
            let position = params.text_document_position.position;
            let offset = document.index.offset(position.line, position.character);
            if let Some(completions) = document
                .analysis
                .option_completions(document.dialect, offset)
            {
                let range = to_range(&document.index, completions.span);
                let items = completions
                    .labels
                    .into_iter()
                    .map(|label| CompletionItem {
                        label: label.to_owned(),
                        kind: Some(CompletionItemKind::FIELD),
                        detail: Some(format!("{} command option", dialect_name(document.dialect))),
                        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                            range,
                            new_text: label.to_owned(),
                        })),
                        ..CompletionItem::default()
                    })
                    .collect();
                return Ok(Some(CompletionResponse::Array(items)));
            }
        }
        let dialect = self.documents.get(uri).map_or_else(
            || Dialect::from_uri_path(uri.path()),
            |document| document.dialect,
        );
        let labels = catalog::commands(dialect).collect::<BTreeSet<_>>();
        let items = labels
            .into_iter()
            .map(|label| CompletionItem {
                label: label.to_owned(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format!("{} command", dialect_name(dialect))),
                documentation: catalog::command_summary(label).map(|summary| {
                    Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: summary.to_owned(),
                    })
                }),
                ..CompletionItem::default()
            })
            .collect();
        Ok(Some(CompletionResponse::Array(items)))
    }
}

fn dialect_name(dialect: Dialect) -> &'static str {
    match dialect {
        Dialect::Tcl => "Tcl",
        Dialect::Sdc => "SDC/Tcl",
        Dialect::Xdc => "XDC/Tcl",
    }
}

fn to_lsp_diagnostics(document: &Document) -> Vec<tower_lsp::lsp_types::Diagnostic> {
    document
        .analysis
        .diagnostics
        .iter()
        .map(|diagnostic| tower_lsp::lsp_types::Diagnostic {
            range: to_range(&document.index, diagnostic.span.clone()),
            severity: Some(match diagnostic.severity {
                Severity::Error => DiagnosticSeverity::ERROR,
                Severity::Warning => DiagnosticSeverity::WARNING,
                Severity::Information => DiagnosticSeverity::INFORMATION,
                Severity::Hint => DiagnosticSeverity::HINT,
            }),
            code: Some(NumberOrString::String(diagnostic.code.to_owned())),
            source: Some("softbrush_ls".to_owned()),
            message: diagnostic.message.clone(),
            ..tower_lsp::lsp_types::Diagnostic::default()
        })
        .collect()
}

fn to_range(index: &LineIndex, span: ByteRange<usize>) -> Range {
    let (start_line, start_character) = index.position(span.start);
    let (end_line, end_character) = index.position(span.end.max(span.start + 1));
    Range::new(
        Position::new(start_line, start_character),
        Position::new(end_line, end_character),
    )
}

fn encode_semantic_tokens(document: &Document) -> Vec<SemanticToken> {
    let mut absolute = Vec::new();
    for semantic in &document.analysis.semantic_spans {
        let mut start = semantic.span.start;
        while start < semantic.span.end {
            let (line, character) = document.index.position(start);
            let mut end = semantic.span.end.min(document.index.line_end(line));
            if end > start && document.text.as_bytes().get(end - 1) == Some(&b'\r') {
                end -= 1;
            }
            if end > start {
                let (_, end_character) = document.index.position(end);
                absolute.push((
                    line,
                    character,
                    end_character - character,
                    semantic_type(semantic.kind),
                    u32::from(semantic.declaration),
                ));
            }
            start = document.index.line_start(line + 1);
            if start == document.text.len() || start >= semantic.span.end {
                break;
            }
        }
    }
    absolute.sort_by_key(|token| (token.0, token.1));

    let mut previous_line = 0;
    let mut previous_start = 0;
    absolute
        .into_iter()
        .map(
            |(line, start, length, token_type, token_modifiers_bitset)| {
                let delta_line = line - previous_line;
                let delta_start = if delta_line == 0 {
                    start - previous_start
                } else {
                    start
                };
                previous_line = line;
                previous_start = start;
                SemanticToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type,
                    token_modifiers_bitset,
                }
            },
        )
        .collect()
}

fn semantic_type(kind: SemanticKind) -> u32 {
    match kind {
        SemanticKind::Comment => 0,
        SemanticKind::String => 1,
        SemanticKind::Number => 2,
        SemanticKind::Variable => 3,
        SemanticKind::Function => 4,
        SemanticKind::Keyword => 5,
        SemanticKind::Operator => 6,
        SemanticKind::Parameter => 7,
        SemanticKind::Namespace => 8,
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Url;

    use super::{Document, encode_semantic_tokens};

    #[test]
    fn semantic_tokens_are_delta_encoded() {
        let uri = Url::parse("file:///constraints/top.sdc").expect("valid URI");
        let document = Document::new(&uri, "# timing\ncreate_clock -period 10\n".to_owned());
        let tokens = encode_semantic_tokens(&document);
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[1].delta_line, 1);
    }
}
