use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use miette::Result;
use ram_parser::parse;
use serde_json::Value;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::info;

use crate::VERSION;

/// Represents a document managed by the LSP server
#[derive(Debug, Clone)]
struct Document {
    uri: Url,
    text: String,
    version: i32,
}

/// Document store to track all open documents
#[derive(Debug, Default)]
struct DocumentStore {
    documents: HashMap<Url, Document>,
}

impl DocumentStore {
    fn new() -> Self {
        Self { documents: HashMap::new() }
    }

    fn get(&self, uri: &Url) -> Option<&Document> {
        self.documents.get(uri)
    }

    fn open(&mut self, uri: Url, text: String, version: i32) {
        self.documents.insert(uri.clone(), Document { uri, text, version });
    }

    fn change(&mut self, uri: &Url, text: String, version: i32) -> bool {
        if let Some(doc) = self.documents.get_mut(uri) {
            doc.text = text;
            doc.version = version;
            true
        } else {
            false
        }
    }

    fn close(&mut self, uri: &Url) -> bool {
        self.documents.remove(uri).is_some()
    }
}

#[derive(Debug)]
struct Backend {
    client: Client,
    document_store: Arc<Mutex<DocumentStore>>,
}

impl Backend {
    fn new(client: Client) -> Self {
        Self { client, document_store: Arc::new(Mutex::new(DocumentStore::new())) }
    }

    /// Parse a document and publish diagnostics
    async fn parse_and_publish_diagnostics(&self, uri: Url, text: String, version: i32) {
        // Parse document and collect errors
        let (_, errors) = parse(&text);

        // Convert errors to LSP diagnostics
        let diagnostics = self.convert_errors_to_diagnostics(&text, errors);

        // Publish diagnostics
        self.client.publish_diagnostics(uri, diagnostics, Some(version)).await;
    }

    /// Convert RAM parser errors to LSP diagnostics
    fn convert_errors_to_diagnostics(
        &self,
        source: &str,
        errors: Vec<ram_parser::ParseError>,
    ) -> Vec<Diagnostic> {
        errors
            .into_iter()
            .map(|err| {
                // Default range
                let range = if let Some((span, _)) = err.labeled_spans.first() {
                    let start_pos = position_at_offset(source, span.start);
                    let end_pos = position_at_offset(source, span.end);
                    Range::new(start_pos, end_pos)
                } else {
                    Range::new(Position::new(0, 0), Position::new(0, 0))
                };

                let mut diagnostic = Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("ram-lsp".to_string()),
                    message: err.message,
                    related_information: None,
                    tags: None,
                    data: None,
                };

                // Add related information for labeled spans
                if err.labeled_spans.len() > 1 {
                    let mut related_info = Vec::new();

                    for (span, label) in err.labeled_spans.iter().skip(1) {
                        let start_pos = position_at_offset(source, span.start);
                        let end_pos = position_at_offset(source, span.end);

                        related_info.push(DiagnosticRelatedInformation {
                            location: Location {
                                uri: Url::parse("file:///")
                                    .unwrap_or_else(|_| Url::parse("about:blank").unwrap()),
                                range: Range::new(start_pos, end_pos),
                            },
                            message: label.clone(),
                        });
                    }

                    if !related_info.is_empty() {
                        diagnostic.related_information = Some(related_info);
                    }
                }

                // Add help text to message
                if !err.help.is_empty() {
                    diagnostic.message = format!("{}\nHelp: {}", diagnostic.message, err.help);
                }

                diagnostic
            })
            .collect()
    }
}

/// Convert a byte offset to LSP Position
fn position_at_offset(text: &str, offset: usize) -> Position {
    let offset = offset.min(text.len());

    let mut line = 0;
    let mut line_start = 0;

    for (i, c) in text.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            line_start = i + 1;
        }
    }

    let character = (offset - line_start) as u32;
    Position::new(line as u32, character)
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        self.client.log_message(MessageType::INFO, "Initializing server").await;
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "RAM Language Server".to_string(),
                version: Some(VERSION.pkg_version().to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        will_save: Some(false),
                        will_save_wait_until: Some(false),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(false),
                        })),
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    ..Default::default()
                }),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["dummy.do_something".to_string()],
                    ..Default::default()
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "RAM language server initialized").await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        self.client.log_message(MessageType::INFO, "Shutting down RAM language server").await;
        Ok(())
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client.log_message(MessageType::INFO, "Workspace folders changed").await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client.log_message(MessageType::INFO, "Configuration changed").await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client.log_message(MessageType::INFO, "Watched files changed").await;
    }

    async fn execute_command(&self, _: ExecuteCommandParams) -> LspResult<Option<Value>> {
        self.client.log_message(MessageType::INFO, "Command executed!").await;
        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        let version = params.text_document.version;

        self.client.log_message(MessageType::INFO, format!("File opened: {}", uri)).await;

        // Store the document (drop the lock before await)
        {
            let mut store = self.document_store.lock().unwrap();
            store.open(uri.clone(), text.clone(), version);
        }

        // Parse document and publish diagnostics
        self.parse_and_publish_diagnostics(uri, text, version).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let version = params.text_document.version;

        // Get the full content from the changes
        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();

            // Store updated document (drop the lock before await)
            let update_success = {
                let mut store = self.document_store.lock().unwrap();
                store.change(&uri, text.clone(), version)
            };

            if !update_success {
                self.client
                    .log_message(MessageType::ERROR, format!("Failed to update document: {}", uri))
                    .await;
                return;
            }

            // Parse document and publish diagnostics
            self.parse_and_publish_diagnostics(uri, text, version).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        self.client.log_message(MessageType::INFO, format!("File saved: {}", uri)).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        // Remove document from store (drop the lock before await)
        let close_success = {
            let mut store = self.document_store.lock().unwrap();
            store.close(&uri)
        };

        if !close_success {
            self.client
                .log_message(
                    MessageType::WARNING,
                    format!("Closed document was not in store: {}", uri),
                )
                .await;
        }

        // Clear diagnostics for closed file
        self.client.publish_diagnostics(uri.clone(), vec![], None).await;
        self.client.log_message(MessageType::INFO, format!("File closed: {}", uri)).await;
    }

    async fn completion(&self, _: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple(
                "LOAD".to_string(),
                "Load a value into the accumulator".to_string(),
            ),
            CompletionItem::new_simple(
                "STORE".to_string(),
                "Store accumulator to memory".to_string(),
            ),
            CompletionItem::new_simple(
                "ADD".to_string(),
                "Add a value to the accumulator".to_string(),
            ),
            CompletionItem::new_simple(
                "SUB".to_string(),
                "Subtract a value from the accumulator".to_string(),
            ),
            CompletionItem::new_simple(
                "MUL".to_string(),
                "Multiply accumulator by a value".to_string(),
            ),
            CompletionItem::new_simple(
                "DIV".to_string(),
                "Divide accumulator by a value".to_string(),
            ),
            CompletionItem::new_simple("JUMP".to_string(), "Jump to a label".to_string()),
            CompletionItem::new_simple(
                "JGTZ".to_string(),
                "Jump if accumulator is greater than zero".to_string(),
            ),
            CompletionItem::new_simple(
                "JZERO".to_string(),
                "Jump if accumulator is zero".to_string(),
            ),
            CompletionItem::new_simple("HALT".to_string(), "Stop execution".to_string()),
        ])))
    }
}

pub async fn run() -> Result<()> {
    info!("Starting LSP server");
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    let (service, socket) = LspService::new(|client| Backend::new(client));

    let server = Server::new(stdin, stdout, socket);

    server.serve(service).await;
    info!("LSP server shutting down");
    Ok(())
}
