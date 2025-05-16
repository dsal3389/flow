use std::borrow::Cow;

use flow_core::Key;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum TokenKind {
    String,
    Number,
    Identifier,

    // keywords
    Bind
}

#[derive(Debug)]
pub(crate) struct Token<'a> {
    literal: Cow<'a, str>,
    line: usize,
    start: usize,
    end: usize,
    kind: TokenKind
}

impl<'a> Token<'a> {
    /// returns the token kind
    pub(crate) fn kind(&self) -> TokenKind {
        self.kind.clone()
    }

    pub(crate) fn literal(&self) -> Cow<'a, str> {
        self.literal.clone()
    }
}

/// iterator type for the Lexer type
#[derive(Debug)]
pub(crate) struct LexerIter<'a> {
    content: &'a str,
    current: usize,
    start: usize,
    line: usize,
}

impl<'a> LexerIter<'a> {
    fn next_token(content: &str) -> Option<(usize, TokenKind)> {
        match content.chars().next().unwrap() {
            'a' .. 'z' | 'A' .. 'Z' | '_' => {
                let identifier = content
                    .chars()
                    .take_while(|c| matches!(c, 'a' .. 'z' | 'A' .. 'Z' | '_'))
                    .collect::<String>();

                if identifier == "bind" {
                    return Some((identifier.len(), TokenKind::Bind))
                }
                todo!()
            }
            _ => todo!()
        }
    }
}

impl<'a> Iterator for LexerIter<'a> {
    type Item = Token<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.content.len() {
            return None;
        }

        let content = &self.content[self.current..];

        match Self::next_token(content) {
            Some((size, kind)) => {
                let start = self.current;
                let end = start + size;
                self.current = end;

                Some(Token {
                    literal: Cow::from(Cow::from(&content[..end])),
                    line: self.line,
                    start,
                    end,
                    kind
                })
            }
            None => None
        }
    }
}

#[derive(Debug)]
pub(crate) struct Lexer {
    content: String,
}

impl Lexer {
    pub(crate) fn new(content: String) -> Lexer {
        Lexer { content }
    }

    /// returns a new type that implements iterator, each iteration
    /// yields new token
    pub(crate) fn iter(&self) -> impl Iterator<Item = Token> {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a Lexer {
    type IntoIter = LexerIter<'a>;
    type Item = Token<'a>;

    fn into_iter(self) -> Self::IntoIter {
        LexerIter {
            content: &self.content,
            current: 0,
            start: 0,
            line: 1,
        }

    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_tokens() {
        let lexer = Lexer::new("bind x y".to_string());
        let mut iter = lexer.iter();

        let token = iter.next().expect("expected first token");
        assert_eq!(token.kind(), TokenKind::Bind);
    }
}
