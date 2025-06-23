// compiler/src/lexer/comments.rs

/// Remove all `//` line comments and `/* ... */` block comments,
/// preserving all other characters (including newlines).
pub fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut chars = src.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    // line comment: consume '//' then skip until newline
                    chars.next();
                    while let Some(&nc) = chars.peek() {
                        if nc == '\n' {
                            break;
                        }
                        chars.next();
                    }
                }
                Some('*') => {
                    // block comment: consume '/*' then skip until '*/'
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' {
                            if let Some(&'/') = chars.peek() {
                                chars.next();
                                break;
                            }
                        }
                    }
                }
                _ => {
                    // not a comment, emit the '/'
                    out.push(c);
                }
            }
        } else {
            out.push(c);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::strip_comments;

    #[test]
    fn removes_line_comments() {
        let src = "let x = 1; // this is x\nlet y = 2; // y\n";
        let out = strip_comments(src);
        assert_eq!(out, "let x = 1; \nlet y = 2; \n");
    }

    #[test]
    fn removes_block_comments_single_line() {
        let src = "let x = /* multi */ 1;\n";
        let out = strip_comments(src);
        assert_eq!(out, "let x =  1;\n");
    }

    #[test]
    fn removes_block_comments_multi_line() {
        let src = "/* start\n still comment */let x = 3;/*end*/\n";
        let out = strip_comments(src);
        assert_eq!(out, "let x = 3;\n");
    }

    #[test]
    fn preserves_non_comment_slashes() {
        let src = "let path = \"/usr/bin\";\nlet div = 10 / 2;\n";
        let out = strip_comments(src);
        assert_eq!(out, src);
    }
}
