///

/// Stmt = Bind(Expr*, Stmt)
///      | Assign(Expr name, Expr value)
///
/// Expr = Literal
///      | Name(Name)
///
/// Literal = String | Char | Number | Pixels
///
///
use std::fmt;

use thiserror::Error;

use crate::lexer::{LexerError, LexerIterContext, Token, TokenKind};

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Error, Debug, Eq, PartialEq)]
pub struct ParseError {
    /// the line number where the parse error happened
    line: usize,
    /// if there are specific colums the error occured in, this is optional since for unexpected EOF
    /// there is no span
    span: Option<(usize, usize)>,
    /// a helpful message that will explain the issue
    message: String,
}

impl From<LexerError> for ParseError {
    fn from(value: LexerError) -> ParseError {
        ParseError {
            line: value.line,
            span: value.span,
            message: value.reason,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.span {
            Some((start, end)) => write!(f, "L#{} {}:{} - {}", self.line, start, end, self.message),
            None => write!(f, "L#{} - {}", self.line, self.message),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Binary {
    left: Expr,
    operator: String,
    right: Expr,
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.operator, self.left, self.right)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Reference {
    name: String,
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'!{}", self.name)
    }
}

#[derive(Debug, Eq, PartialEq)]
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

#[derive(Debug, Eq, PartialEq)]
struct Assign {
    name: String,
    value: Expr,
}

impl fmt::Display for Assign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(assign {} {})", self.name, self.value)
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Name {
    name: String,
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}", self.name)
    }
}

#[derive(Debug, Eq, PartialEq)]
enum Expr {
    Name(Name),
    Binary(Box<Binary>),
    Literal(Literal),
    Assign(Box<Assign>),
    Reference(Reference),
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

    #[inline]
    pub(crate) fn is_reference(&self) -> bool {
        matches!(self, Self::Reference(_))
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v: &dyn fmt::Display = match self {
            Self::Name(inner) => inner,
            Self::Binary(inner) => inner,
            Self::Literal(inner) => inner,
            Self::Assign(inner) => inner,
            Self::Reference(inner) => inner,
        };
        write!(f, "{}", v)
    }
}

/// like `Block` but doesn't expect opening and closing
/// brackets
#[derive(Debug, Eq, PartialEq)]
struct Body {
    stmts: Vec<Stmt>,
}

#[derive(Debug)]
enum BindAction {
    Spawn { target: String, args: Vec<String> },
}

#[derive(Debug, Eq, PartialEq)]
struct BindStmt {
    exprs: Vec<Expr>,
    stmt: Stmt,
}

#[derive(Debug, Eq, PartialEq)]
enum Stmt {
    Bind(Box<BindStmt>),
    Expr(Expr),
    Empty,
}

/// a simple macro that is used only by the `Parser` to check
/// the kind of the token, if the token is not of the expected kind, a `ParseError`
/// is returned, if the token is of the expected kind, the token is returned, this
/// ```rs
/// // assume `next` returns an Option<Result<Token, LexerError>>
/// let token = self.next().and_then(|token| expect_token_kind!(token?, TokenKind::LeftParen))?;
/// ```
macro_rules! expect_token_kind {
    ($token:expr, $kinds:pat $(,)?) => {
        expect_token_kind!($token, $kinds, "unexpected next token".to_string())
    };
    ($token:expr, $kinds:pat $(,)?, $err_message:expr) => {
        match $token.kind() {
            $kinds => Ok($token),
            _ => Err(ParseError {
                line: $token.line(),
                span: Some($token.span()),
                message: $err_message,
            }),
        }
    };
}

pub(crate) struct Parser<I>
where
    I: LexerIterContext,
{
    iter: I,
    peeked: Option<Result<Token>>,
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

    /// returns the next token, if None is returned
    /// it means there is no next token
    #[inline]
    fn next(&mut self) -> Option<Result<Token>> {
        match self.peeked.take() {
            Some(token) => Some(token),
            None => self.iter.next().map(|next| next.map_err(|err| err.into())),
        }
    }

    /// return the next token, but returns an error if
    /// there is no next token
    #[inline]
    fn expect_next(&mut self) -> Result<Result<Token>> {
        match self.next() {
            Some(token) => Ok(token),
            None => Err(ParseError {
                span: None,
                line: self.iter.line(),
                message: "unexpected eof".to_string(),
            }),
        }
    }

    /// peeks to the next token, returns None
    /// if there is no next value
    #[inline]
    fn peek(&mut self) -> Option<&Result<Token>> {
        if self.peeked.is_some() {
            self.peeked.as_ref()
        } else {
            self.peeked = self.next();
            self.peeked.as_ref()
        }
    }

    #[inline]
    fn eof(&mut self) -> bool {
        self.peek().is_none()
    }

    pub fn parse(&mut self) -> Result<Body> {
        let mut stmts = Vec::new();
        while !self.eof() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Body { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        let _ = self
            .expect_next()?
            .and_then(|token| expect_token_kind!(token, TokenKind::LeftParen))?;

        // the first token in the statement should tell us
        // what is the kind of the statement, and we call the correct
        // parser based on the given statement
        let stmt = match self.expect_next()??.kind() {
            TokenKind::RightParen => return Ok(Stmt::Empty),
            TokenKind::Bind => self.parse_bind_stmt(),
            _ => todo!(),
        };

        let _ = self
            .expect_next()?
            .and_then(|token| expect_token_kind!(token, TokenKind::RightParen))?;
        stmt
    }

    fn parse_bind_stmt(&mut self) -> Result<Stmt> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_ast_display() {
        let ast = Expr::Binary(Box::new(Binary {
            left: Expr::Literal(Literal::Number(55)),
            operator: "*".to_string(),
            right: Expr::Literal(Literal::Number(77)),
        }));
        assert_eq!(format!("{}", ast), "(* 55 77)")
    }

    #[test]
    fn test_ast_empty_stmt() {
        let lexer = Lexer::new("()".to_string());
        let mut parser = Parser::new(&lexer);
        assert_eq!(
            parser.parse(),
            Ok(Body {
                stmts: vec![Stmt::Empty]
            })
        );
    }

    #[test]
    fn test_ast_bind_stmt() {
        let lexer = Lexer::new("(bind x + 'z ())".to_string());
        let mut parser = Parser::new(&lexer);

        assert_eq!(
            parser.parse(),
            Ok(Body {
                stmts: vec![Stmt::Bind(Box::new(BindStmt {
                    exprs: vec![
                        Expr::Name(Name {
                            name: "x".to_string()
                        }),
                        Expr::Name(Name {
                            name: "z".to_string()
                        })
                    ],
                    stmt: Stmt::Empty
                }))]
            })
        );
    }
}
