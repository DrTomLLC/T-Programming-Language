use crate::handlers::definition::handle_definition;

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    // …
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>, tower_lsp::jsonrpc::Error> {
                Ok(None);
                    handle_definition(self, params).await
    }

    async fn completion(
            &self,
            params: CompletionParams,
        ) -> Result<Option<CompletionResponse>, tower_lsp::jsonrpc::Error> {
            handle_completion(self, params).await
    }
}