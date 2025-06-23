// compiler/src/lexer/normalize.rs

/// Convert any `\r\n` (Windows) into `\n`. Leaves lone `\r` intact.
/// This makes line‐counting, lookups, and comments stripping consistent.
pub fn normalize_line_endings(src: &str) -> String {
    src.replace("\r\n", "\n")
}

#[cfg(test)]
mod tests {
    use super::normalize_line_endings;

    #[test]
    fn keeps_unix_intact() {
        let src = "a\nb\nc\n";
        assert_eq!(normalize_line_endings(src), src);
    }

    #[test]
    fn converts_windows() {
        let src = "a\r\nb\r\n";
        assert_eq!(normalize_line_endings(src), "a\nb\n");
    }

    #[test]
    fn mixed_line_breaks() {
        let src = "x\r\ny\nz\r\n";
        assert_eq!(normalize_line_endings(src), "x\ny\nz\n");
    }
}
