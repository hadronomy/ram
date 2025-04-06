use chumsky::prelude::*;
#[cfg(feature = "serde")]
use serde::Serialize;

pub type Span = SimpleSpan;
pub type Spanned<T> = (T, Span);

#[derive(Debug, Clone, PartialEq)]
enum Token<'src> {
    Ident(&'src str),
    Num(u64),
    Str(&'src str),
    Op(&'src str),
    Ctrl(char),
    Comment(&'src str),
}

impl<'src> std::fmt::Display for Token<'src> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(s) => write!(f, "Ident({s})"),
            Token::Num(n) => write!(f, "Num({n})"),
            Token::Str(s) => write!(f, "Str({s})"),
            Token::Op(s) => write!(f, "Op({s})"),
            Token::Ctrl(c) => write!(f, "Ctrl({c})"),
        }
    }
}

pub fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char, Span>>> {
    let num = text::int(10)
        .then(just('.').then(text::digits(10)).or_not())
        .to_slice()
        .from_str()
        .unwrapped()
        .map(Token::Num);

    let op = one_of("=*").repeated().at_least(1).to_slice().map(Token::Op);

    let ctrl = one_of("()[]{};,").map(Token::Ctrl);

    let ident = text::ident().map(Token::Ident);

    let str = just('"')
        .ignore_then(filter(|c: &char| *c != '"').repeated())
        .then_ignore(just('"'))
        .to_slice()
        .map(Token::Str);

    let comment = just('#')
        .ignore_then(filter(|c: &char| *c != '\n').repeated())
        .to_slice()
        .map(Token::Comment);

    comment.or(num).or(op).or(ctrl).or(ident).map_with_span(|tok, span| (tok, span))
}

pub fn parser() -> impl Parser<char, Program, Error = Simple<char>> {
    program_parser()
}
