#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    String(String),
    True,
    False,

    // Identifiers
    Ident(String),

    // Keywords
    Let,
    Fn,
    If,
    Else,
    Return,
    While,

    // Operators
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,

    // Comparison
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Logical
    And,
    Or,

    // Delimiters
    Comma,
    Semicolon,
    Colon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // Special
    Eof,
}
