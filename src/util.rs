//! Utility functions and helpers.
//!
//! Shared helpers for ID generation, formatting invocation, etc.

use std::process::Command;
use thiserror::Error;

/// Errors that can occur during formatting.
#[derive(Debug, Error)]
pub enum FormatError {
    #[error("rustfmt not found in PATH")]
    RustfmtNotFound,

    #[error("rustfmt failed: {0}")]
    RustfmtFailed(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Format Rust code using rustfmt.
///
/// Returns the formatted code, or the original code with a warning if rustfmt fails.
pub fn format_rust_code(code: &str) -> Result<String, FormatError> {
    // Check if rustfmt is available
    let rustfmt_check = Command::new("rustfmt").arg("--version").output();

    if rustfmt_check.is_err() {
        return Err(FormatError::RustfmtNotFound);
    }

    // Run rustfmt
    let mut child = Command::new("rustfmt")
        .arg("--emit=stdout")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    // Write code to stdin
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        stdin.write_all(code.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(FormatError::RustfmtFailed(stderr.to_string()))
    }
}

/// Try to format code, returning original on failure.
pub fn try_format_rust_code(code: &str) -> String {
    match format_rust_code(code) {
        Ok(formatted) => formatted,
        Err(e) => {
            eprintln!("Warning: Could not format code: {}", e);
            code.to_string()
        }
    }
}

/// Validate that a string is a valid Rust identifier.
pub fn is_valid_rust_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();

    // First character must be a letter or underscore
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    // Remaining characters must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// List of Rust keywords that cannot be used as identifiers.
pub const RUST_KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true",
    "type", "unsafe", "use", "where", "while",
];

/// Check if a string is a Rust keyword.
pub fn is_rust_keyword(s: &str) -> bool {
    RUST_KEYWORDS.contains(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_rust_identifier() {
        assert!(is_valid_rust_identifier("foo"));
        assert!(is_valid_rust_identifier("_bar"));
        assert!(is_valid_rust_identifier("foo_bar"));
        assert!(is_valid_rust_identifier("foo123"));
        assert!(is_valid_rust_identifier("_"));

        assert!(!is_valid_rust_identifier(""));
        assert!(!is_valid_rust_identifier("123foo"));
        assert!(!is_valid_rust_identifier("foo-bar"));
        assert!(!is_valid_rust_identifier("foo bar"));
    }

    #[test]
    fn test_is_rust_keyword() {
        assert!(is_rust_keyword("fn"));
        assert!(is_rust_keyword("struct"));
        assert!(is_rust_keyword("let"));
        assert!(!is_rust_keyword("foo"));
        assert!(!is_rust_keyword("myStruct"));
    }

    #[test]
    fn test_is_valid_rust_identifier_unicode() {
        // ASCII only for identifiers
        assert!(!is_valid_rust_identifier("föo"));
        assert!(!is_valid_rust_identifier("名前"));
    }

    #[test]
    fn test_is_valid_rust_identifier_edge_cases() {
        assert!(is_valid_rust_identifier("_0"));
        assert!(is_valid_rust_identifier("A"));
        assert!(is_valid_rust_identifier("z"));
        assert!(is_valid_rust_identifier("___"));
        assert!(!is_valid_rust_identifier("0_"));
    }

    #[test]
    fn test_try_format_rust_code() {
        let code = "fn main() { println!(\"hello\"); }";
        let result = try_format_rust_code(code);
        // Should either be formatted or return original
        assert!(result.contains("fn main"));
    }

    #[test]
    fn test_rust_keywords_comprehensive() {
        // Test a few more keywords
        assert!(is_rust_keyword("async"));
        assert!(is_rust_keyword("await"));
        assert!(is_rust_keyword("dyn"));
        assert!(is_rust_keyword("impl"));
        assert!(is_rust_keyword("Self"));
        assert!(is_rust_keyword("super"));
        assert!(is_rust_keyword("crate"));
    }
}
