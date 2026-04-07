use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use gobject_lint::ast_context::AstContext;
use gobject_lint::config::Config;
use gobject_lint::scanner;

pub struct GObjectBackend {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, String>>>,
}

impl GObjectBackend {
    pub fn new(client: Client) -> Self {
        GObjectBackend {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn lint_document(&self, uri: &Url) -> Result<()> {
        let path = uri
            .to_file_path()
            .map_err(|_| tower_lsp::jsonrpc::Error::invalid_params("Invalid file path"))?;

        // Get workspace root (parent directory of the file)
        let workspace_root = path.parent().unwrap_or(&path).to_path_buf();

        // Load config from workspace root
        let config_path = workspace_root.join("gobject-lint.toml");
        let config = if config_path.exists() {
            match Config::load(&config_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to load config: {}", e);
                    return Ok(());
                }
            }
        } else {
            // Create empty config
            Config {
                ignore: Vec::new(),
                rules: Default::default(),
                editor_url: None,
            }
        };

        // Build ignore matcher
        let ignore_matcher = match config.build_ignore_matcher() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to build ignore matcher: {}", e);
                return Ok(());
            }
        };

        // Build AST context for the workspace
        let ast_context =
            match AstContext::build_with_ignore(&workspace_root, &ignore_matcher, None) {
                Ok(ctx) => ctx,
                Err(e) => {
                    eprintln!("Failed to build AST context: {}", e);
                    return Ok(());
                }
            };

        // Run scanner
        let violations = match scanner::scan_with_ast(&ast_context, &config, &workspace_root, None)
        {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to scan: {}", e);
                return Ok(());
            }
        };

        // Convert violations to diagnostics
        let diagnostics: Vec<Diagnostic> = violations
            .iter()
            .filter(|v| v.file == path)
            .map(|v| {
                let range = Range {
                    start: Position {
                        line: v.line.saturating_sub(1) as u32,
                        character: v.column.saturating_sub(1) as u32,
                    },
                    end: Position {
                        line: v.line.saturating_sub(1) as u32,
                        character: v.column.saturating_sub(1) as u32 + 1,
                    },
                };

                Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::WARNING),
                    code: Some(NumberOrString::String(v.rule.to_string())),
                    source: Some("gobject-lint".to_string()),
                    message: v.message.clone(),
                    ..Default::default()
                }
            })
            .collect();

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, None)
            .await;
        Ok(())
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for GObjectBackend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "GObject LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.documents.lock().await.insert(uri.clone(), text);
        let _ = self.lint_document(&uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(change) = params.content_changes.into_iter().next() {
            self.documents.lock().await.insert(uri.clone(), change.text);
            let _ = self.lint_document(&uri).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let _ = self.lint_document(&params.text_document.uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents
            .lock()
            .await
            .remove(&params.text_document.uri);
    }
}
