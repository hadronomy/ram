use std::sync::{Arc, Mutex, RwLock};

use miette::Result;
use ram_diagnostics::{Diagnostic, DiagnosticKind};
use serde_json::Value;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::{debug, error, info};
use url::Url;

use crate::db::FileId;

mod db;
mod highlighting;

use crate::db::LspDatabase;
use crate::highlighting::{
    semantic_tokens_for_tree, semantic_tokens_legend, to_lsp_semantic_tokens,
};

/// The version of the LSP server
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The restart command ID
const RESTART_COMMAND: &str = "ram.server.restart";

#[derive(Debug)]
struct Backend {
    /// The LSP client
    client: Client,
    /// The database for the LSP server
    db: Arc<RwLock<LspDatabase>>,
    /// Flag to indicate if the server should restart
    should_restart: Arc<Mutex<bool>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> LspResult<InitializeResult> {
        self.client.log_message(MessageType::INFO, "Initializing RAM Language Server").await;

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "RAM Language Server".to_string(),
                version: Some(VERSION.to_string()),
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
                    commands: vec![RESTART_COMMAND.to_string()],
                    ..Default::default()
                }),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensRegistrationOptions(
                        SemanticTokensRegistrationOptions {
                            text_document_registration_options: TextDocumentRegistrationOptions {
                                document_selector: None,
                            },
                            semantic_tokens_options: SemanticTokensOptions {
                                work_done_progress_options: WorkDoneProgressOptions::default(),
                                legend: semantic_tokens_legend(),
                                range: Some(true),
                                full: Some(SemanticTokensFullOptions::Bool(true)),
                            },
                            static_registration_options: StaticRegistrationOptions::default(),
                        },
                    ),
                ),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "RAM Language Server initialized").await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        self.client.log_message(MessageType::INFO, "Shutting down RAM Language Server").await;
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

    async fn execute_command(&self, params: ExecuteCommandParams) -> LspResult<Option<Value>> {
        match params.command.as_str() {
            RESTART_COMMAND => {
                self.client.log_message(MessageType::INFO, "Restarting server...").await;

                // Set the restart flag - drop the lock before await
                {
                    let mut should_restart = self.should_restart.lock().unwrap();
                    *should_restart = true;
                }

                // Show a message to the user
                self.client
                    .show_message(MessageType::INFO, "RAM Language Server is restarting...")
                    .await;

                Ok(None)
            }
            _ => {
                self.client
                    .log_message(
                        MessageType::WARNING,
                        format!("Unknown command: {}", params.command),
                    )
                    .await;
                Ok(None)
            }
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        debug!("File opened: {}", uri);

        // Add the file to the database
        let file_id = {
            let mut db = self.db.write().unwrap();
            db.add_file(uri.clone(), &text)
        };

        // Publish diagnostics
        self.publish_diagnostics(file_id, uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        debug!("File changed: {}", uri);

        // Get the file ID
        let file_id = {
            let db = self.db.read().unwrap();
            match db.file_id_for_url(&uri) {
                Some(id) => id,
                None => {
                    error!("File not found in database: {}", uri);
                    return;
                }
            }
        };

        // Apply the changes
        {
            let mut db = self.db.write().unwrap();

            // Get the current text
            let current_text = match db.file_text(file_id) {
                Some(text) => text,
                None => {
                    error!("File text not found for file ID: {:?}", file_id);
                    return;
                }
            };

            // Apply the changes to get the new text
            let mut new_text = current_text;
            for change in params.content_changes {
                if let Some(range) = change.range {
                    // Convert LSP range to string indices
                    let start_pos = position_to_index(&new_text, range.start);
                    let end_pos = position_to_index(&new_text, range.end);

                    // Apply the change
                    new_text.replace_range(start_pos..end_pos, &change.text);
                } else {
                    // Full document update
                    new_text = change.text;
                }
            }

            // Update the file in the database
            db.add_file(uri.clone(), &new_text);
        }

        // Publish diagnostics
        self.publish_diagnostics(file_id, uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        debug!("File saved: {}", uri);

        // Get the file ID
        let file_id = {
            let db = self.db.read().unwrap();
            match db.file_id_for_url(&uri) {
                Some(id) => id,
                None => {
                    error!("File not found in database: {}", uri);
                    return;
                }
            }
        };

        // If text is provided, update the file
        if let Some(text) = params.text {
            let mut db = self.db.write().unwrap();
            db.add_file(uri.clone(), &text);
        }

        // Publish diagnostics
        self.publish_diagnostics(file_id, uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        debug!("File closed: {}", uri);

        // Clear diagnostics for the file
        self.client.publish_diagnostics(uri.clone(), vec![], None).await;
    }

    async fn completion(&self, _: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        // Basic completion for RAM instructions
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple(
                "LOAD".to_string(),
                "Load a value into the accumulator".to_string(),
            ),
            CompletionItem::new_simple(
                "STORE".to_string(),
                "Store the accumulator value in memory".to_string(),
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
                "Multiply the accumulator by a value".to_string(),
            ),
            CompletionItem::new_simple(
                "DIV".to_string(),
                "Divide the accumulator by a value".to_string(),
            ),
            CompletionItem::new_simple("JMP".to_string(), "Jump to a label".to_string()),
            CompletionItem::new_simple(
                "JGTZ".to_string(),
                "Jump if the accumulator is greater than zero".to_string(),
            ),
            CompletionItem::new_simple(
                "JZERO".to_string(),
                "Jump if the accumulator is zero".to_string(),
            ),
            CompletionItem::new_simple("HALT".to_string(), "Halt the program".to_string()),
            CompletionItem::new_simple("READ".to_string(), "Read a value from input".to_string()),
            CompletionItem::new_simple("WRITE".to_string(), "Write a value to output".to_string()),
        ])))
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> LspResult<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri;

        // Get the file ID and syntax tree - clone what we need to avoid holding locks across await points
        let syntax_tree = {
            let db = self.db.read().unwrap();
            let file_id = match db.file_id_for_url(&uri) {
                Some(id) => id,
                None => {
                    error!("File not found in database: {}", uri);
                    return Ok(None);
                }
            };

            match db.syntax_tree_for_file(file_id) {
                Some(tree) => tree.clone(),
                None => {
                    error!("Syntax tree not found for file: {}", uri);
                    return Ok(None);
                }
            }
        };

        // Get semantic tokens
        let tokens = semantic_tokens_for_tree(&syntax_tree);
        let lsp_tokens = to_lsp_semantic_tokens(tokens);

        Ok(Some(SemanticTokensResult::Tokens(lsp_tokens)))
    }

    async fn semantic_tokens_range(
        &self,
        params: SemanticTokensRangeParams,
    ) -> LspResult<Option<SemanticTokensRangeResult>> {
        // For simplicity, we'll just return the full tokens
        // In a real implementation, you would filter tokens by range
        let tokens = match self
            .semantic_tokens_full(SemanticTokensParams {
                text_document: params.text_document,
                work_done_progress_params: Default::default(),
                partial_result_params: PartialResultParams::default(),
            })
            .await?
        {
            Some(SemanticTokensResult::Tokens(tokens)) => tokens,
            _ => return Ok(None),
        };

        Ok(Some(SemanticTokensRangeResult::Tokens(tokens)))
    }
}

impl Backend {
    /// Publish diagnostics for a file
    async fn publish_diagnostics(&self, file_id: FileId, uri: Url) {
        // Get the diagnostics and file text from the database
        // We need to clone the data we need so we don't hold the lock across await points
        let (diagnostics, file_text) = {
            let db = self.db.read().unwrap();
            let diags = match db.diagnostics_for_file(file_id) {
                Some(diags) => diags.clone(),
                None => {
                    debug!("No diagnostics found for file: {}", uri);
                    return;
                }
            };

            let text = match db.file_text(file_id) {
                Some(text) => text.to_string(),
                None => {
                    error!("File text not found for file ID: {:?}", file_id);
                    return;
                }
            };

            (diags, text)
        };

        // Convert to LSP diagnostics
        let lsp_diagnostics = diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| convert_diagnostic_to_lsp(&file_text, diagnostic))
            .collect::<Vec<_>>();

        // Publish the diagnostics
        self.client.publish_diagnostics(uri, lsp_diagnostics, None).await;
    }
}

/// Convert a position to an index in the text
fn position_to_index(text: &str, position: Position) -> usize {
    let mut line = 0;
    let mut column = 0;
    let mut index = 0;

    for c in text.chars() {
        if line == position.line as usize && column == position.character as usize {
            return index;
        }

        if c == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }

        index += c.len_utf8();
    }

    index
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

/// Convert a diagnostic to an LSP diagnostic
fn convert_diagnostic_to_lsp(
    source: &str,
    diagnostic: &Diagnostic,
) -> tower_lsp::lsp_types::Diagnostic {
    // Get the primary span
    let primary_span = diagnostic.labeled_spans.first().cloned().unwrap_or((0..0, "".to_string()));

    let file_text = source;

    // Convert the span to an LSP range
    let range = Range {
        start: position_at_offset(file_text, primary_span.0.start),
        end: position_at_offset(file_text, primary_span.0.end),
    };

    // Convert the diagnostic kind to an LSP severity
    let severity = match diagnostic.kind {
        DiagnosticKind::Error => Some(DiagnosticSeverity::ERROR),
        DiagnosticKind::Warning => Some(DiagnosticSeverity::WARNING),
        DiagnosticKind::Advice => Some(DiagnosticSeverity::INFORMATION),
        DiagnosticKind::Custom(_) => Some(DiagnosticSeverity::ERROR),
    };

    // Create related information for secondary spans
    let related_information = if diagnostic.labeled_spans.len() > 1 {
        let related_info = diagnostic
            .labeled_spans
            .iter()
            .skip(1)
            .map(|(span, label)| DiagnosticRelatedInformation {
                location: Location {
                    uri: Url::parse("file:///")
                        .unwrap_or_else(|_| Url::parse("about:blank").unwrap()),
                    range: Range {
                        start: position_at_offset(file_text, span.start),
                        end: position_at_offset(file_text, span.end),
                    },
                },
                message: label.clone(),
            })
            .collect::<Vec<_>>();

        if !related_info.is_empty() { Some(related_info) } else { None }
    } else {
        None
    };

    // Create the LSP diagnostic
    let mut message = diagnostic.message.clone();

    // Add help text to message if available
    if !diagnostic.help.is_empty() {
        message = format!("{}\nHelp: {}", message, diagnostic.help);
    }

    // Add notes to message if available
    if !diagnostic.notes.is_empty() {
        for note in &diagnostic.notes {
            message = format!("{}\nNote: {}", message, note);
        }
    }

    tower_lsp::lsp_types::Diagnostic {
        range,
        severity,
        code: diagnostic.code.clone().map(NumberOrString::String),
        code_description: None,
        source: Some("ram-lsp".to_string()),
        message,
        related_information,
        tags: None,
        data: None,
    }
}

/// Run the LSP server
pub async fn run() -> Result<()> {
    // Use a loop to handle server restarts
    loop {
        info!("Starting RAM Language Server");
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

        // Create the database
        let db = Arc::new(RwLock::new(LspDatabase::new()));

        // Create the restart flag
        let should_restart = Arc::new(Mutex::new(false));

        // Create the service
        let (service, socket) = LspService::new(|client| Backend {
            client,
            db: Arc::clone(&db),
            should_restart: Arc::clone(&should_restart),
        });

        // Create the server
        let server = Server::new(stdin, stdout, socket);

        // Run the server
        server.serve(service).await;

        // Check if we should restart
        let restart = *should_restart.lock().unwrap();
        if !restart {
            info!("RAM Language Server shutting down");
            break;
        }

        info!("Restarting RAM Language Server");
    }

    Ok(())
}
