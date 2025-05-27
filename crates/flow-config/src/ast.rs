///
/// Stmt = Bind(Expr*, Stmt)
///      | Assign(Expr name, Expr value)
///
/// Expr = Literal
///      | Reference(String)
///
/// Literal = String | Char | Number | Pixels
///
///
use std::fmt;

use thiserror::Error;

use crate::lexer::{LexerIterContext, Token, TokenKind};

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Error, Debug)]
pub struct ParseError {
    /// the line number where the parse error happened
    line: usize,
    /// if there are specific colums the error occured in, this is optional since for unexpected EOF
    /// there is no span
    span: Option<(usize, usize)>,
    /// a helpful message that will explain the issue
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.span {
            Some((start, end)) => write!(f, "L#{} {}:{} - {}", self.line, start, end, self.message),
            None => write!(f, "L#{} - {}", self.line, self.message),
        }
    }
}

#[derive(Debug)]
struct Binary {
    left: Expr,
    operator: Token,
    right: Expr,
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({} {} {})",
            self.operator.literal(),
            self.left,
            self.right
        )
    }
}

#[derive(Debug)]
struct Reference {
    name: String
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'!{}", self.name)
    }
}

#[derive(Debug)]
enum Literal {
    String(String),
    Pixels(u32),
    Number(i32),
    Char(char),
}

impl Literal {
    #[inline]
    pub(crate) fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    #[inline]
    pub(crate) fn is_pixels(&self) -> bool {
        matches!(self, Self::Pixels(_))
    }

    #[inline]
    pub(crate) fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    #[inline]
    pub(crate) fn is_char(&self) -> bool {
        matches!(self, Self::Char(_))
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v = match self {
            Self::String(s) => format!("\"{}\"", s),
            Self::Pixels(p) => format!("{}px", p),
            Self::Number(n) => n.to_string(),
            Self::Char(c) => c.to_string(),
        };
        write!(f, "{}", v)
    }
}

#[derive(Debug)]
struct Assign {
    name: String,
    value: Expr,
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(assign {} {})", self.name, self.value)
    }
}

#[derive(Debug)]
enum Expr {
    Binary(Box<Binary>),
    Literal(Literal),
    Assign(Box<Assign>),
    Reference(Reference)
}

impl Expr {
    #[inline]
    pub(crate) fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    #[inline]
    pub(crate) fn is_assignment(&self) -> bool {
        matches!(self, Self::Assign(_))
    }

    #[inline]
    pub(crate) fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v: &dyn fmt::Display = match self {
            Self::Binary(inner) => inner,
            Self::Literal(inner) => inner,
            Self::Assign(inner) => inner,
            Self::Reference(inner) => inner
        };
        write!(f, "{}", v)
    }
}

/// like `Block` but doesn't expect opening and closing
/// brackets
#[derive(Debug)]
struct Body {
    stmts: Vec<Stmt>,
}

#[derive(Debug)]
enum BindAction {
    Spawn { target: String, args: Vec<String> },
}

#[derive(Debug)]
struct BindStmt {
    exprs: Vec<Expr>,
    stmt: Stmt,
}

#[derive(Debug)]
enum Stmt {
    Bind(Box<BindStmt>),
    Expr(Expr),
}

pub(crate) struct Parser<I>
where
    I: LexerIterContext,
{
    iter: I,
    peeked: Option<I::Item>,
}

impl<I> Parser<I>
where
    I: LexerIterContext,
{
    pub fn new<T>(value: T) -> Parser<I>
    where
        T: IntoIterator<IntoIter = I, Item = I::Item>,
    {
        Parser {
            iter: value.into_iter(),
            peeked: None,
        }
    }

    pub fn parse(&mut self) -> Result<Body> {
        let mut stmts = Vec::new();
        while !self.eof() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Body { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        self.check_next(
            TokenKind::LeftParen,
            "expected open paren `(` to parse statement",
        )?;

        let stmt = match self.expect_next(None)?.kind() {
            TokenKind::Bind => {
                let bind_stmt = Box::new(self.parse_bind_stmt()?);
                Stmt::Bind(bind_stmt)
            }
            _ => todo!(),
        };

        self.check_next(
            TokenKind::RightParen,
            "expected a closing paren `)` for statement"
        )?;
        Ok(stmt)
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        todo!()
    }

    fn parse_bind_stmt(&mut self) -> Result<BindStmt> {
        let mut exprs = Vec::new();

        while !self.eof() {
            let expr = self.parse_expr().and_then(|expr| {
                if expr.is_literal() {
                    todo!()
                }
                todo!()
            })?;

            exprs.push(expr);

            // the next value is peeked because if the token is
            // of type LeftParen we don't want to consume
            if let Some(token) = self.peek() {
                match token.kind() {
                    TokenKind::LeftParen => break,
                    TokenKind::Plus => {
                        self.next();
                        continue;
                    },
                    _ => return Err(ParseError {
                        line: token.line(),
                        span: Some(token.span()),
                        message: "unexpected token while parsing bind statement, expected a `+` or start of another block".to_string(),
                    })
                }
            }
        }

        let stmt = self.parse_stmt()?;
        Ok(BindStmt { exprs, stmt })
    }

    /// returnes the next item from the iterator
    fn next(&mut self) -> Option<I::Item> {
        if self.peeked.is_some() {
            self.peeked.take()
        } else {
            self.iter.next()
        }
    }

    /// returns a boolean value indicating if the next token
    /// is of the given type, if there is no next token `false` will be returned
    #[inline]
    fn is_next(&mut self, kind: TokenKind) -> bool {
        self.peek().is_some_and(|token| matches!(token.kind(), kind))
    }

    /// tries to take the next token, if there is no token or the token is not what
    /// was expected, a ParseError is returned
    #[must_use]
    fn check_next(&mut self, kind: TokenKind, message: &str) -> Result<I::Item> {
        self.expect_next(Some(message)).and_then(|token| {
            if matches!(&token, kind) {
                Ok(token)
            } else {
             Err(ParseError {
                line: token.line(),
                span: Some(token.span()),
                message: message.to_string(),
            })
            }
        })
    }

    /// like regular next but returns an error if there is no
    /// next value, appends also the given message to the error info
    #[must_use]
    fn expect_next(&mut self, message: Option<&str>) -> Result<I::Item> {
        self.next().ok_or(ParseError {
            span: None,
            line: self.iter.line(),
            message: format!("unexpected EOF {}", message.unwrap_or_default())
        })
    }

    /// peeks the next element in the iterator and returns it, if there is nothing to peek
    /// then None is returned
    fn peek(&mut self) -> Option<&I::Item> {
        if self.peeked.is_none() {
            self.peeked = self.iter.next();
        }
        self.peeked.as_ref()
    }

    /// returns a boolean value indicating if the
    /// parser has reached the end and there are no more tokens to process
    #[inline]
    fn eof(&mut self) -> bool {
        self.peek().is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_visitor() {
        let ast = Expr::Binary(Box::new(Binary {
            left: Expr::Literal(Literal::Number(55)),
            operator: Token::new("*".to_string(), 1, 0, 0, TokenKind::Star),
            right: Expr::Literal(Literal::Number(77)),
        }));
        assert_eq!(format!("{}", ast), "(* 55 77)")
    }
}
