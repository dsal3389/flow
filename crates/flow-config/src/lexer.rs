use std::borrow::Cow;


#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) enum TokenKind {
    Dot,
    Comma,
    WhiteSpace,

    // literals
    String,
    Number,
    Identifier,

    // keywords
    Bind,
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    literal: String,
    line: usize,
    start: usize,
    end: usize,
    kind: TokenKind,
}

impl Token {
    /// returns the token kind
    pub(crate) fn kind(&self) -> TokenKind {
        self.kind.clone()
    }

    /// returns the token literal value
    pub(crate) fn literal(&self) -> &str {
        &self.literal
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
    /// returns the first token from the given string and the
    /// token literal length
    fn next_token_from_string(content: &str) -> Option<(usize, TokenKind)> {
        if content.is_empty() {
            return None;
        }

        // since we already checked that the string is not empty
        // it is safe to unwrap and expect at least 1 char
        match content.chars().next().unwrap() {
            '.' => Some((1, TokenKind::Dot)),
            ',' => Some((1, TokenKind::Comma)),
            ' ' | '\r' | '\n' => {
                let count = content
                    .chars()
                    .take_while(|c| matches!(c, ' ' | '\r' | '\n'))
                    .count();
                Some((count, TokenKind::WhiteSpace))
            }
            '0'..'9' => {
                let count = content
                    .chars()
                    .take_while(|c| matches!(c, '0'..'9'))
                    .count();
                Some((count, TokenKind::Number))
            }
            'a'..'z' | 'A'..'Z' | '_' => {
                let identifier = content
                    .chars()
                    .take_while(|c| matches!(c, 'a' .. 'z' | 'A' .. 'Z' | '_'))
                    .collect::<String>();

                // TODO:  optimize with some hash table or something
                let token_kind = match identifier.as_str() {
                    "bind" => TokenKind::Bind,
                    _ => TokenKind::Identifier,
                };
                Some((identifier.len(), token_kind))
            }
            _ => todo!(),
        }
    }
}

impl<'a> Iterator for LexerIter<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.content.len() {
            return None;
        }

        let content = &self.content[self.current..];

        match Self::next_token_from_string(content) {
            Some((size, kind)) => {
                let start = self.current;
                let end = start + size;
                self.current = end;

                Some(Token {
                    literal: content[..end].to_string(),
                    line: self.line,
                    start,
                    end,
                    kind,
                })
            }
            None => None,
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
    type Item = Token;

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
    fn test_lexer_bind_parse() {
        let lexer = Lexer::new("bind x y".to_string());
        let token = lexer.iter().next().expect("expected first token");
        assert_eq!(token.kind(), TokenKind::Bind);
    }

    #[test]
    fn test_lexer_int_parse() {
        let lexer = Lexer::new("777 text".to_string());
        let token = lexer.iter().next().expect("expected first token");
        assert_eq!(token.kind(), TokenKind::Number);
        assert_eq!(token.literal(), "777")
    }

    #[test]
    fn test_lexer_identifier_parse() {
        let lexer = Lexer::new("identifier".to_string());
        let token = lexer.iter().next().expect("expected first token");
        assert_eq!(token.kind(), TokenKind::Identifier);
        assert_eq!(token.literal(), "identifier");
    }
}
