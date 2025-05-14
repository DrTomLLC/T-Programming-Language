use compiler::runtime::eval::Interpreter;
pub use compiler::parser::expr::parse;
use shared::tokenizer::tokenize;

pub fn run_source(source: &str) -> Result<String, String> {
    let tokens = tokenize(source).map_err(|e| e.to_string())?;
    let statements = parse(tokens).map_err(|e| e.to_string())?;

    let mut eval = Interpreter::new();
    let mut out = String::new();

    for stmt in statements {
        let val = eval.eval_stmt(stmt, 0).map_err(|e| e.to_string())?;
        out.push_str(&format!("{:?}\n", val)); // Use Debug until Display is implemented
    }

    Ok(out)
}
