use std::collections::HashMap;
use std::sync::Arc;

use jmespath_extensions::Runtime;
use jmespath_extensions::registry::FunctionRegistry;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

struct JmespathLsp {
    client: Client,
    registry: Arc<FunctionRegistry>,
    documents: Arc<RwLock<HashMap<Url, String>>>,
}

impl JmespathLsp {
    fn new(client: Client) -> Self {
        let mut registry = FunctionRegistry::new();
        registry.register_all();
        Self {
            client,
            registry: Arc::new(registry),
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get completions for function names
    fn get_function_completions(&self) -> Vec<CompletionItem> {
        self.registry
            .functions()
            .map(|func| {
                let documentation = Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "{}\n\n**Category:** {}\n\n**Signature:** `{}`",
                        func.description,
                        func.category.name(),
                        func.signature
                    ),
                });

                CompletionItem {
                    label: func.name.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(func.signature.to_string()),
                    documentation: Some(documentation),
                    insert_text: Some(format!("{}($0)", func.name)),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..Default::default()
                }
            })
            .collect()
    }

    /// Get hover info for a function name
    fn get_function_hover(&self, name: &str) -> Option<Hover> {
        self.registry.get_function(name).map(|func| {
            let mut content = format!("## {}\n\n", func.name);
            content.push_str(func.description);
            content.push_str("\n\n");
            content.push_str(&format!("**Signature:** `{}`\n\n", func.signature));
            content.push_str(&format!("**Category:** {}\n", func.category.name()));

            if !func.aliases.is_empty() {
                content.push_str(&format!("\n**Aliases:** {}", func.aliases.join(", ")));
            }

            if let Some(jep) = func.jep {
                content.push_str(&format!("\n**JEP:** {}", jep));
            }

            if !func.example.is_empty() {
                content.push_str(&format!(
                    "\n\n**Example:**\n```jmespath\n{}\n```",
                    func.example
                ));
            }

            Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: content,
                }),
                range: None,
            }
        })
    }

    /// Parse expression and return diagnostics
    fn get_diagnostics(&self, text: &str) -> Vec<Diagnostic> {
        let mut runtime = Runtime::new();
        runtime.register_builtin_functions();
        jmespath_extensions::register_all(&mut runtime);

        match runtime.compile(text) {
            Ok(_) => vec![],
            Err(e) => {
                // Extract position from error
                let line = e.line as u32;
                let col = e.column as u32;

                vec![Diagnostic {
                    range: Range {
                        start: Position {
                            line,
                            character: col,
                        },
                        end: Position {
                            line,
                            character: col + 1,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    source: Some("jmespath".to_string()),
                    message: e.to_string(),
                    ..Default::default()
                }]
            }
        }
    }

    /// Extract word at position from text
    fn word_at_position(text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        let line = lines.get(position.line as usize)?;
        let col = position.character as usize;

        if col > line.len() {
            return None;
        }

        // Find word boundaries
        let start = line[..col]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = line[col..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| col + i)
            .unwrap_or(line.len());

        if start < end {
            Some(line[start..end].to_string())
        } else {
            None
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for JmespathLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "jpx-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "JMESPath LSP initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = self.get_function_completions();
        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let documents = self.documents.read().await;
        if let Some(text) = documents.get(&uri)
            && let Some(word) = Self::word_at_position(text, position)
        {
            return Ok(self.get_function_hover(&word));
        }

        Ok(None)
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();

        // Store document
        {
            let mut documents = self.documents.write().await;
            documents.insert(uri.clone(), text.clone());
        }

        // Publish diagnostics
        let diagnostics = self.get_diagnostics(&text);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        if let Some(change) = params.content_changes.first() {
            let text = change.text.clone();

            // Update stored document
            {
                let mut documents = self.documents.write().await;
                documents.insert(uri.clone(), text.clone());
            }

            // Publish diagnostics
            let diagnostics = self.get_diagnostics(&text);
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        // Remove document
        {
            let mut documents = self.documents.write().await;
            documents.remove(&uri);
        }

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(JmespathLsp::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
