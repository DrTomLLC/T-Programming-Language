use shared::ast::*;

pub trait ASTVisitor {
    fn visit_ast(&mut self, ast: &AST) {
        for item in &ast.items {
            self.visit_item(item);
        }
    }

    fn visit_item(&mut self, item: &Item) {
        match item {
            Item::Module { items, .. } => {
                for sub in items { self.visit_item(sub) }
            }
            // recurse into signatures, expressions, etc…
            _ => {}
        }
    }
}

pub trait ASTTransformer {
    fn transform_ast(&mut self, ast: AST) -> AST {
        AST {
            items: ast.items.into_iter().map(|i| self.transform_item(i)).collect(),
            ..ast
        }
    }

    fn transform_item(&mut self, item: Item) -> Item {
        match item {
            Item::Module { name, attrs, items, span } => {
                let items = items.into_iter().map(|i| self.transform_item(i)).collect();
                Item::Module { name, attrs, items, span }
            }
            // clone‑and‑recurse every variant…
            other => other,
        }
    }
}
