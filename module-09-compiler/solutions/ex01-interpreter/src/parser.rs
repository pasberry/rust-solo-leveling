use crate::ast::*;
use crate::error::{ParseError, ParseResult};
use crate::lexer::Lexer;
use crate::token::Token;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    LogicalOr,      // ||
    LogicalAnd,     // &&
    Equals,         // ==, !=
    LessGreater,    // <, >, <=, >=
    Sum,            // +, -
    Product,        // *, /
    Prefix,         // -x, !x
    Call,           // fn(x)
    Index,          // array[index]
}

fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Or => Precedence::LogicalOr,
        Token::And => Precedence::LogicalAnd,
        Token::Eq | Token::NotEq => Precedence::Equals,
        Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => Precedence::LessGreater,
        Token::Plus | Token::Minus => Precedence::Sum,
        Token::Star | Token::Slash => Precedence::Product,
        Token::LParen => Precedence::Call,
        Token::LBracket => Precedence::Index,
        _ => Precedence::Lowest,
    }
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }

    pub fn parse_program(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while self.current_token != Token::Eof {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> ParseResult<Stmt> {
        match &self.current_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::While => self.parse_while_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> ParseResult<Stmt> {
        self.expect_token(Token::Let)?;

        let name = match &self.current_token {
            Token::Ident(s) => s.clone(),
            _ => return Err(ParseError::ExpectedIdentifier),
        };
        self.advance();

        self.expect_token(Token::Assign)?;

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.current_token == Token::Semicolon {
            self.advance();
        }

        Ok(Stmt::Let { name, value })
    }

    fn parse_return_statement(&mut self) -> ParseResult<Stmt> {
        self.expect_token(Token::Return)?;

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.current_token == Token::Semicolon {
            self.advance();
        }

        Ok(Stmt::Return(value))
    }

    fn parse_while_statement(&mut self) -> ParseResult<Stmt> {
        self.expect_token(Token::While)?;

        let condition = self.parse_expression(Precedence::Lowest)?;

        self.expect_token(Token::LBrace)?;
        let body = self.parse_block_statement()?;

        Ok(Stmt::While { condition, body })
    }

    fn parse_expression_statement(&mut self) -> ParseResult<Stmt> {
        // Check if this is an assignment (identifier = expression)
        if let Token::Ident(name) = &self.current_token.clone() {
            let name = name.clone();
            self.advance();

            if self.current_token == Token::Assign {
                self.advance();
                let value = self.parse_expression(Precedence::Lowest)?;

                if self.current_token == Token::Semicolon {
                    self.advance();
                }

                return Ok(Stmt::Assign { name, value });
            } else {
                // Not an assignment, backtrack and parse as expression
                // Put the identifier back and parse normally
                let expr = if self.current_token == Token::LParen {
                    // Function call
                    let func = Expr::Identifier(name);
                    self.parse_infix(func)?
                } else if self.current_token == Token::LBracket {
                    // Array/hash index
                    let left = Expr::Identifier(name);
                    self.parse_infix(left)?
                } else if self.is_infix_token(&self.current_token) {
                    // Infix expression
                    let left = Expr::Identifier(name);
                    self.parse_infix(left)?
                } else {
                    // Just an identifier
                    Expr::Identifier(name)
                };

                if self.current_token == Token::Semicolon {
                    self.advance();
                }

                return Ok(Stmt::Expression(expr));
            }
        }

        // Not an identifier, parse as normal expression statement
        let expr = self.parse_expression(Precedence::Lowest)?;

        if self.current_token == Token::Semicolon {
            self.advance();
        }

        Ok(Stmt::Expression(expr))
    }

    fn is_infix_token(&self, token: &Token) -> bool {
        matches!(
            token,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Eq
                | Token::NotEq
                | Token::Lt
                | Token::Gt
                | Token::LtEq
                | Token::GtEq
                | Token::And
                | Token::Or
                | Token::LParen
                | Token::LBracket
        )
    }

    fn parse_expression(&mut self, precedence: Precedence) -> ParseResult<Expr> {
        let mut left = self.parse_prefix()?;

        while self.current_token != Token::Semicolon
            && self.current_token != Token::Eof
            && precedence < self.current_precedence()
        {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> ParseResult<Expr> {
        match &self.current_token.clone() {
            Token::Integer(n) => {
                let expr = Expr::Integer(*n);
                self.advance();
                Ok(expr)
            }
            Token::True => {
                self.advance();
                Ok(Expr::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Boolean(false))
            }
            Token::String(s) => {
                let expr = Expr::String(s.clone());
                self.advance();
                Ok(expr)
            }
            Token::Ident(name) => {
                let expr = Expr::Identifier(name.clone());
                self.advance();
                Ok(expr)
            }
            Token::Bang | Token::Minus => self.parse_prefix_expression(),
            Token::LParen => self.parse_grouped_expression(),
            Token::LBracket => self.parse_array_literal(),
            Token::LBrace => self.parse_hash_literal(),
            Token::If => self.parse_if_expression(),
            Token::Fn => self.parse_function_literal(),
            _ => Err(ParseError::UnexpectedToken(format!("{:?}", self.current_token))),
        }
    }

    fn parse_prefix_expression(&mut self) -> ParseResult<Expr> {
        let operator = match &self.current_token {
            Token::Bang => PrefixOp::Bang,
            Token::Minus => PrefixOp::Minus,
            _ => return Err(ParseError::InvalidOperator),
        };

        self.advance();

        let right = self.parse_expression(Precedence::Prefix)?;

        Ok(Expr::Prefix {
            operator,
            right: Box::new(right),
        })
    }

    fn parse_grouped_expression(&mut self) -> ParseResult<Expr> {
        self.expect_token(Token::LParen)?;

        let expr = self.parse_expression(Precedence::Lowest)?;

        self.expect_token(Token::RParen)?;

        Ok(expr)
    }

    fn parse_array_literal(&mut self) -> ParseResult<Expr> {
        self.expect_token(Token::LBracket)?;

        let elements = self.parse_expression_list(Token::RBracket)?;

        Ok(Expr::Array(elements))
    }

    fn parse_hash_literal(&mut self) -> ParseResult<Expr> {
        self.expect_token(Token::LBrace)?;

        let mut pairs = Vec::new();

        while self.current_token != Token::RBrace {
            let key = self.parse_expression(Precedence::Lowest)?;

            self.expect_token(Token::Colon)?;

            let value = self.parse_expression(Precedence::Lowest)?;

            pairs.push((key, value));

            if self.current_token == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }

        self.expect_token(Token::RBrace)?;

        Ok(Expr::Hash(pairs))
    }

    fn parse_if_expression(&mut self) -> ParseResult<Expr> {
        self.expect_token(Token::If)?;

        let condition = Box::new(self.parse_expression(Precedence::Lowest)?);

        self.expect_token(Token::LBrace)?;
        let consequence = self.parse_block_statement()?;

        let alternative = if self.current_token == Token::Else {
            self.advance();
            self.expect_token(Token::LBrace)?;
            Some(self.parse_block_statement()?)
        } else {
            None
        };

        Ok(Expr::If {
            condition,
            consequence,
            alternative,
        })
    }

    fn parse_function_literal(&mut self) -> ParseResult<Expr> {
        self.expect_token(Token::Fn)?;
        self.expect_token(Token::LParen)?;

        let parameters = self.parse_function_parameters()?;

        self.expect_token(Token::LBrace)?;
        let body = self.parse_block_statement()?;

        Ok(Expr::Function { parameters, body })
    }

    fn parse_function_parameters(&mut self) -> ParseResult<Vec<String>> {
        let mut params = Vec::new();

        if self.current_token == Token::RParen {
            self.advance();
            return Ok(params);
        }

        match &self.current_token {
            Token::Ident(name) => {
                params.push(name.clone());
                self.advance();
            }
            _ => return Err(ParseError::ExpectedIdentifier),
        }

        while self.current_token == Token::Comma {
            self.advance();

            match &self.current_token {
                Token::Ident(name) => {
                    params.push(name.clone());
                    self.advance();
                }
                _ => return Err(ParseError::ExpectedIdentifier),
            }
        }

        self.expect_token(Token::RParen)?;

        Ok(params)
    }

    fn parse_block_statement(&mut self) -> ParseResult<Vec<Stmt>> {
        let mut statements = Vec::new();

        while self.current_token != Token::RBrace && self.current_token != Token::Eof {
            statements.push(self.parse_statement()?);
        }

        self.expect_token(Token::RBrace)?;

        Ok(statements)
    }

    fn parse_infix(&mut self, left: Expr) -> ParseResult<Expr> {
        match &self.current_token {
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Eq
            | Token::NotEq
            | Token::Lt
            | Token::Gt
            | Token::LtEq
            | Token::GtEq
            | Token::And
            | Token::Or => self.parse_infix_expression(left),
            Token::LParen => self.parse_call_expression(left),
            Token::LBracket => self.parse_index_expression(left),
            _ => Ok(left),
        }
    }

    fn parse_infix_expression(&mut self, left: Expr) -> ParseResult<Expr> {
        let operator = self.token_to_infix_op(&self.current_token)?;
        let precedence = self.current_precedence();
        self.advance();

        let right = self.parse_expression(precedence)?;

        Ok(Expr::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_call_expression(&mut self, function: Expr) -> ParseResult<Expr> {
        self.expect_token(Token::LParen)?;
        let arguments = self.parse_expression_list(Token::RParen)?;

        Ok(Expr::Call {
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_index_expression(&mut self, left: Expr) -> ParseResult<Expr> {
        self.expect_token(Token::LBracket)?;

        let index = self.parse_expression(Precedence::Lowest)?;

        self.expect_token(Token::RBracket)?;

        Ok(Expr::Index {
            left: Box::new(left),
            index: Box::new(index),
        })
    }

    fn parse_expression_list(&mut self, end: Token) -> ParseResult<Vec<Expr>> {
        let mut list = Vec::new();

        if self.current_token == end {
            self.advance();
            return Ok(list);
        }

        list.push(self.parse_expression(Precedence::Lowest)?);

        while self.current_token == Token::Comma {
            self.advance();
            list.push(self.parse_expression(Precedence::Lowest)?);
        }

        self.expect_token(end)?;

        Ok(list)
    }

    fn advance(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn expect_token(&mut self, expected: Token) -> ParseResult<()> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(format!(
                "expected {:?}, got {:?}",
                expected, self.current_token
            )))
        }
    }

    fn current_precedence(&self) -> Precedence {
        token_precedence(&self.current_token)
    }

    fn token_to_infix_op(&self, token: &Token) -> ParseResult<InfixOp> {
        match token {
            Token::Plus => Ok(InfixOp::Plus),
            Token::Minus => Ok(InfixOp::Minus),
            Token::Star => Ok(InfixOp::Multiply),
            Token::Slash => Ok(InfixOp::Divide),
            Token::Eq => Ok(InfixOp::Equal),
            Token::NotEq => Ok(InfixOp::NotEqual),
            Token::Lt => Ok(InfixOp::LessThan),
            Token::Gt => Ok(InfixOp::GreaterThan),
            Token::LtEq => Ok(InfixOp::LessThanEqual),
            Token::GtEq => Ok(InfixOp::GreaterThanEqual),
            Token::And => Ok(InfixOp::And),
            Token::Or => Ok(InfixOp::Or),
            _ => Err(ParseError::InvalidOperator),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let_statement() {
        let input = "let x = 5;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        match &program[0] {
            Stmt::Let { name, value } => {
                assert_eq!(name, "x");
                assert_eq!(value, &Expr::Integer(5));
            }
            _ => panic!("Expected Let statement"),
        }
    }

    #[test]
    fn test_parse_return_statement() {
        let input = "return 10;";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        match &program[0] {
            Stmt::Return(expr) => {
                assert_eq!(expr, &Expr::Integer(10));
            }
            _ => panic!("Expected Return statement"),
        }
    }

    #[test]
    fn test_parse_infix_expression() {
        let input = "5 + 10 * 2";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        // Should parse as: 5 + (10 * 2) due to precedence
    }

    #[test]
    fn test_parse_function() {
        let input = "fn(x, y) { x + y }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();

        assert_eq!(program.len(), 1);
        match &program[0] {
            Stmt::Expression(Expr::Function { parameters, .. }) => {
                assert_eq!(parameters, &vec!["x".to_string(), "y".to_string()]);
            }
            _ => panic!("Expected function"),
        }
    }
}
