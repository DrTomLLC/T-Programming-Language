// compiler/src/parser/entry.rs - Fixed parse_stmt method

/// Parse one statement. Dispatches to the comprehensive statement parser.
fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
    use crate::parser::statement::parse_statement;
    parse_statement(self).map_err(|e| ParseError::Custom(e.to_string()))
}