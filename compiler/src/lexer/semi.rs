// compiler/src/lexer/semi.rs

/// Go through each line of `src` and, if a non-blank line
/// doesn’t already end in `;`, `{`, or `}`, append a `;`.
pub fn insert_semicolons(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    for line in src.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.is_empty() {
            // preserve blank lines
            out.push_str(line);
            out.push('\n');
            continue;
        }
        // Look at last non-whitespace character
        let last_char = trimmed_end.chars().last().unwrap();
        if last_char == ';' || last_char == '{' || last_char == '}' {
            // already terminated appropriately
            out.push_str(line);
        } else {
            // append semicolon after trimmed content
            out.push_str(trimmed_end);
            out.push(';');
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::insert_semicolons;

    #[test]
    fn adds_semicolon_to_simple_statements() {
        let src = "let x = 1\nlet y = x + 2";
        let out = insert_semicolons(src);
        assert_eq!(out, "let x = 1;\nlet y = x + 2;\n");
    }

    #[test]
    fn preserves_existing_semicolons_and_braces() {
        let src = r#"
fn foo() {
    let a = 3;
}
"#;
        let out = insert_semicolons(src);
        // blank first line, `fn foo() {` ends with `{`, inner line ends with `;`, `}` ends with `}`
        assert_eq!(out, "\nfn foo() {\n    let a = 3;\n}\n");
    }

    #[test]
    fn mixed_whitespace_lines() {
        let src = "   \n  let z = 5  \n";
        let out = insert_semicolons(src);
        assert_eq!(out, "   \n  let z = 5;\n\n");
    }
}
