use std::fmt;

use thiserror::Error;
use crate::lexer::{Token, TokenKind};


#[derive(Error, Debug)]
pub enum ParseError {
    UnexpectedToken {
        message: String,
        token: Token
    },
    MissingToken {
        message: String
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::MissingToken { message } => write!(f, "{}", message),
            ParseError::UnexpectedToken { message, token } => {
                let (start, end) = token.span();
                write!(f, "{} {}:{} {}", token.line(), start, end, message)
            }
        }
    }
}

#[derive(Debug)]
struct Binary {
    left: Expr,
    operator: Token,
    right: Expr
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.operator.literal(), self.left, self.right)
    }
}

#[derive(Debug)]
struct Literal {
    token: Token
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.token.literal())
    }
}

#[derive(Debug)]
struct Assign {
    name: String,
    value: Token,
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(assign {} {})", self.name, self.value.literal())
    }
}

#[derive(Debug)]
enum Expr {
    Binary(Box<Binary>),
    Literal(Literal),
    Assign(Assign)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Binary(inner) => write!(f, "{}", inner),
            Self::Literal(inner) => write!(f, "{}", inner),
            Self::Assign(inner) => write!(f, "{}", inner)
        }
    }
}

/// like `Block` but doesn't expect opening and closing
/// brackets
#[derive(Debug)]
struct Body {
    stmts: Vec<Stmt>
}

#[derive(Debug)]
struct Block {
    stmts: Vec<Stmt>
}

#[derive(Debug)]
enum IfStmt {
    If {
        condition: Expr,
        body: Block
    },
    ElseIf {
        condition: Expr,
        body: Block
    },
    Else {
        body: Block
    }
}

#[derive(Debug)]
enum Stmt {
    If(IfStmt),
    Block(Block),
    Expr(Expr),
}

pub(crate) struct Parser<I>
where
    I: Iterator<Item = Token>
{
    iter: std::iter::Peekable<I>
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>
{
    pub fn new<T>(value: T) -> Parser<I>
    where
        T: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        Parser {
            iter: value.into_iter().peekable()
        }
    }

    pub fn parse(&mut self) -> Result<Body, ParseError> {
        todo!()
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        todo!()
    }

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect_token(TokenKind::LeftBrace, "expect opening braces")?;

        let mut closed = false;
        let mut stmts = Vec::new();

        while let Some(token) = self.iter.next() {
            if matches!(token.kind(), TokenKind::RightBrace) {
                closed = true;
                break;
            }
            let stmt = self.parse_stmt()?;
            stmts.push(stmt);
        }

        if closed {
            Ok(Block { stmts })
        } else {
            // token is expected to be } but it is also a missing token
            // need to rethink the ParseError structure
            todo!()
        }
    }

    fn parse_if_stmt(&mut self) -> Result<IfStmt, ParseError> {
        todo!()
    }

    fn expect_token(&mut self, kind: TokenKind, message: &str) -> Result<Token, ParseError> {
        let token = self.iter.next()
            .ok_or(ParseError::MissingToken { message: message.to_string() })?;

        if matches!(&token, kind) {
            Ok(token)
        } else {
            Err(ParseError::UnexpectedToken {
                message: message.to_string(),
                token
            })
        }
    }

    fn consume_token_if<F>(&mut self, cond: F) -> Option<Token>
    where
        F: FnOnce(&Token) -> bool
    {
        todo!()
    }

    /// returns a boolean value indicating if the
    /// parser has reached the end and there are no more tokens to process
    fn eof(&mut self) -> bool {
        self.iter.peek().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_visitor() {
        let ast = Expr::Binary(Box::new(Binary {
            left: Expr::Literal(Literal {
                token: Token::new("55".to_string(), 1, 0, 0, TokenKind::Number),
            }),
            operator: Token::new("*".to_string(), 1, 0, 0, TokenKind::Star),
            right: Expr::Literal(Literal {
                token: Token::new("77".to_string(), 1, 0, 0, TokenKind::Number),
            })
        }));
        assert_eq!(format!("{}", ast), "(* 55 77)")
    }
}
