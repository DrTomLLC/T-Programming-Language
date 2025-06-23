// compiler/src/lexer/shebang.rs

/// If the very first two bytes of `src` are `#!`, strip until the first `\n`.
/// Otherwise returns the original string slice.
pub fn strip_shebang(src: &str) -> &str {
    if src.starts_with("#!") {
        // find end of first line
        if let Some(idx) = src.find('\n') {
            &src[idx + 1..]
        } else {
            // whole file was just a shebang
            ""
        }
    } else {
        src
    }
}

#[cfg(test)]
mod tests {
    use super::strip_shebang;

    #[test]
    fn removes_shebang_line() {
        let src = "#!/usr/bin/env t\nlet x = 1;\n";
        assert_eq!(strip_shebang(src), "let x = 1;\n");
    }

    #[test]
    fn keeps_no_shebang() {
        let src = "let x = 1;\n";
        assert_eq!(strip_shebang(src), src);
    }

    #[test]
    fn only_shebang() {
        let src = "#!nothing here";
        assert_eq!(strip_shebang(src), "");
    }
}
