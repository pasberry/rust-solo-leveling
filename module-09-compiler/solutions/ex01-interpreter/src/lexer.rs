use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();

        Lexer {
            input: chars,
            position: 0,
            current_char,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current_char {
            None => Token::Eof,
            Some(ch) => match ch {
                '+' => {
                    self.advance();
                    Token::Plus
                }
                '-' => {
                    self.advance();
                    Token::Minus
                }
                '*' => {
                    self.advance();
                    Token::Star
                }
                '/' => {
                    self.advance();
                    if self.current_char == Some('/') {
                        self.skip_comment();
                        self.next_token()
                    } else {
                        Token::Slash
                    }
                }
                '=' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::Eq
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::NotEq
                    } else {
                        Token::Bang
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::LtEq
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::GtEq
                    } else {
                        Token::Gt
                    }
                }
                '&' if self.peek() == Some('&') => {
                    self.advance();
                    self.advance();
                    Token::And
                }
                '|' if self.peek() == Some('|') => {
                    self.advance();
                    self.advance();
                    Token::Or
                }
                '(' => {
                    self.advance();
                    Token::LParen
                }
                ')' => {
                    self.advance();
                    Token::RParen
                }
                '{' => {
                    self.advance();
                    Token::LBrace
                }
                '}' => {
                    self.advance();
                    Token::RBrace
                }
                '[' => {
                    self.advance();
                    Token::LBracket
                }
                ']' => {
                    self.advance();
                    Token::RBracket
                }
                ',' => {
                    self.advance();
                    Token::Comma
                }
                ';' => {
                    self.advance();
                    Token::Semicolon
                }
                ':' => {
                    self.advance();
                    Token::Colon
                }
                '"' => self.read_string(),
                _ if ch.is_ascii_digit() => self.read_number(),
                _ if ch.is_ascii_alphabetic() || ch == '_' => self.read_identifier(),
                _ => {
                    self.advance();
                    panic!("Unexpected character: {}", ch);
                }
            },
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        Token::Integer(num_str.parse().unwrap())
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let ident: String = self.input[start..self.position].iter().collect();

        match ident.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            "while" => Token::While,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Ident(ident),
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance(); // Skip opening quote
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch == '"' {
                let s: String = self.input[start..self.position].iter().collect();
                self.advance(); // Skip closing quote
                return Token::String(s);
            }
            self.advance();
        }

        panic!("Unterminated string");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic_operators() {
        let input = "5 + 10 - 3 * 2 / 4";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Integer(5));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Integer(10));
        assert_eq!(lexer.next_token(), Token::Minus);
        assert_eq!(lexer.next_token(), Token::Integer(3));
        assert_eq!(lexer.next_token(), Token::Star);
        assert_eq!(lexer.next_token(), Token::Integer(2));
        assert_eq!(lexer.next_token(), Token::Slash);
        assert_eq!(lexer.next_token(), Token::Integer(4));
        assert_eq!(lexer.next_token(), Token::Eof);
    }

    #[test]
    fn test_comparison_operators() {
        let input = "== != < > <= >=";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Eq);
        assert_eq!(lexer.next_token(), Token::NotEq);
        assert_eq!(lexer.next_token(), Token::Lt);
        assert_eq!(lexer.next_token(), Token::Gt);
        assert_eq!(lexer.next_token(), Token::LtEq);
        assert_eq!(lexer.next_token(), Token::GtEq);
    }

    #[test]
    fn test_logical_operators() {
        let input = "&& ||";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::And);
        assert_eq!(lexer.next_token(), Token::Or);
    }

    #[test]
    fn test_keywords() {
        let input = "let fn if else return while true false";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Let);
        assert_eq!(lexer.next_token(), Token::Fn);
        assert_eq!(lexer.next_token(), Token::If);
        assert_eq!(lexer.next_token(), Token::Else);
        assert_eq!(lexer.next_token(), Token::Return);
        assert_eq!(lexer.next_token(), Token::While);
        assert_eq!(lexer.next_token(), Token::True);
        assert_eq!(lexer.next_token(), Token::False);
    }

    #[test]
    fn test_string_literal() {
        let input = r#""hello world""#;
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::String("hello world".to_string()));
    }

    #[test]
    fn test_comment() {
        let input = "5 + 10 // this is a comment\n20";
        let mut lexer = Lexer::new(input);

        assert_eq!(lexer.next_token(), Token::Integer(5));
        assert_eq!(lexer.next_token(), Token::Plus);
        assert_eq!(lexer.next_token(), Token::Integer(10));
        assert_eq!(lexer.next_token(), Token::Integer(20));
    }
}
