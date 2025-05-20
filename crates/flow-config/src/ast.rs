use std::borrow::Cow;
use std::num::ParseIntError;

use thiserror::Error;

use crate::lexer::{Lexer, Token, TokenKind};
use flow_core::Key;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Requirement {
    Required,
    NotRequired
}

#[derive(Error, Debug)]
pub enum TokenNodeError {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error("issue parsing token, expected next token ")]
    ExpectedNextToken {
        expected: TokenKind,
        found: Token
    }
}

#[derive(Debug, Clone)]
pub struct FlowNumber {
    value: usize,
}

#[derive(Debug, Clone)]
pub struct FlowString {
    value: String,
}

#[derive(Debug, Clone)]
pub struct FlowKey {
    key: Key,
}

#[derive(Debug, Clone)]
pub enum FlowAction {
    Spawn { prog: String, args: Vec<String> },
}

#[derive(Debug, Clone)]
pub struct FlowBind<'a> {
    key: Cow<'a, FlowKey>,
    action: FlowAction,
}

pub enum TokenNode {
    Body(Vec<TokenNode>),
    Key(FlowKey),
    String(FlowString),
    Number(FlowNumber),
}

impl TokenNode {
    fn try_from_lexer(lexer: Lexer) -> Result<Vec<TokenNode>, TokenNodeError> {
        let mut nodes = Vec::new();
        let mut lex_iter = lexer.iter();

        while let Some(token) = lex_iter.next() {
            match token.kind() {
                TokenKind::Bind => {
                    todo!()
                }
                _ => todo!()
            }
        }
        Ok(nodes)
    }
}

pub struct AstRoot(TokenNode);

impl AstRoot {
    pub fn from_lexer<'a>(lex: Lexer) -> Result<AstRoot, TokenNodeError> {
        let nodes = TokenNode::try_from_lexer(lex)?;
        Ok(AstRoot(TokenNode::Body(nodes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
