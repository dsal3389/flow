use crate::lexer::{Lexer, Token, TokenKind};
use flow_core::Key;


trait Visitor<T> {
    type Ret;
    fn visit(&self, value: &T) -> Self::Ret;
}

trait VisitorAccept {
    fn accept<V>(&self, visitor: &V) -> V::Ret
    where
        V: Visitor<Self>,
        Self: Sized
    {
        visitor.visit(self)
    }
}

pub struct PrintVisitor;

impl Visitor<Expr> for PrintVisitor {
    type Ret = String;
    fn visit(&self, value: &Expr) -> Self::Ret {
        match value {
            Expr::Literal(inner) => inner.accept(self),
            Expr::Binary(inner) => inner.accept(self),
            _ => todo!()
        }
    }
}

impl Visitor<Binary> for PrintVisitor {
    type Ret = String;
    fn visit(&self, value: &Binary) -> Self::Ret {
        let mut buffer = String::with_capacity(16);
        buffer.push('(');
        buffer.push_str(value.operator.literal());
        buffer.push(' ');
        buffer.push_str(&value.left.accept(self));
        buffer.push(' ');
        buffer.push_str(&value.right.accept(self));
        buffer.push(')');
        buffer
    }
}

impl Visitor<Literal> for PrintVisitor {
    type Ret = String;
    fn visit(&self, value: &Literal) -> Self::Ret {
        value.token.literal().into()
    }
}

#[derive(Debug)]
struct Binary {
    left: Expr,
    operator: Token,
    right: Expr
}

#[derive(Debug)]
struct Literal {
    token: Token
}

#[derive(Debug)]
enum Expr {
    Binary(Box<Binary>),
    Literal(Literal)
}

impl VisitorAccept for Binary {}
impl VisitorAccept for Literal {}
impl VisitorAccept for Expr {}


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
        assert_eq!(ast.accept(&PrintVisitor), "(* 55 77)");
    }
}
