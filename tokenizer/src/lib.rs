use logos::Filter;
use logos::{Lexer, Logos, Skip};
use std::collections::VecDeque;
use std::env;
use std::fs;

fn newline_cb(lex: &mut Lexer<LexToken>) -> usize {
    lex.span().len() - 1
}

/// Compute the line and column position for the current word.
fn word_callback(lex: &mut Lexer<LexToken>) -> (usize, usize) {
    (0, 0)
}

#[derive(Default)]
pub struct Extras {
    indents: Vec<usize>,
}

impl Extras {
    pub fn indent(&self) -> usize {
        *self.indents.last().unwrap_or(&0)
    }

    pub fn push_indent(&mut self, indent: usize) {
        self.indents.push(indent);
    }

    pub fn pop_indent(&mut self) {
        self.indents.pop();
    }
}

#[derive(Logos, Copy, Clone, Eq, PartialEq, Debug)]
#[logos(extras = Extras)]
#[logos(skip r"[ \t]+")]
pub enum LexToken {
    #[regex(r";[^\n]*", logos::skip)]
    Comment,

    #[regex(r"FIRRTL version \d+\.\d+\.\d+")]
    Version,

    #[regex(r"\n( )*", newline_cb)]
    Newline(usize),

    #[token(r"circuit")]
    KwCircuit,
    #[token(r"module")]
    KwModule,
    #[token(r"skip")]
    KwSkip,
    #[token(r"input")]
    KwInput,
    #[token(r"output")]
    KwOutput,

    #[regex(r"(\w+|`[^`]+`)")]
    Id,

    #[regex(r"@\[[^\]]*]")]
    Info,

    #[token(r"{")]
    CurlyLeft,
    #[token(r"}")]
    CurlyRight,
    #[token(r"[")]
    BracketLeft,
    #[token(r"]")]
    BracketRight,
    #[token(r"(")]
    ParenLeft,
    #[token(r")")]
    ParenRight,
    #[token(r"<")]
    AngLeft,
    #[token(r">")]
    AngRight,

    #[token(r",")]
    Comma,
    #[token(r":")]
    Colon,
    #[token(r"=")]
    Eq,
    #[token(r".")]
    Dot,

    #[regex(r#""([^"]|\\")*""#)]
    String,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Token {
    Lex(LexToken),
    Newline,
    Indent,
    Dedent,
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Lex(lex_token) => write!(f, "{lex_token:?}"),
            Token::Newline => write!(f, "Newline"),
            Token::Indent => write!(f, "Indent"),
            Token::Dedent => write!(f, "Dedent"),
        }
    }
}

pub struct FirrtlLexer<'a> {
    lex: Lexer<'a, LexToken>,
    token_queue: VecDeque<Token>,
    indents: Vec<usize>,
}

impl<'a> FirrtlLexer<'a> {
    pub fn new(s: &str) -> FirrtlLexer {
        let mut lex = LexToken::lexer(s);
        FirrtlLexer {
            lex,
            token_queue: VecDeque::with_capacity(4),
            indents: vec![],
        }
    }

    pub fn indent_level(&self) -> usize {
        *self.indents.last().unwrap_or(&0)
    }

    fn redent(&mut self, level: usize) {
        if level > self.indent_level() {
            self.indents.push(level);
            self.token_queue.push_back(Token::Indent);
        } else {
            while level < self.indent_level() {
                self.indents.pop();
                self.token_queue.push_back(Token::Dedent);
            }
        }
    }

    pub fn next(&mut self) -> Option<Result<Token, ()>> {
        if let Some(token) = self.token_queue.pop_front() {
            return Some(Ok(token));
        } else {
            let maybe_token = self.lex.next() ;
            match maybe_token {
                Some(Ok(LexToken::Newline(indent))) => {
                    self.redent(indent);
                    Some(Ok(Token::Newline))
                }
                Some(Ok(tok)) => Some(Ok(Token::Lex(tok))),
                Some(Err(e)) => {
                    dbg!(e);
                    dbg!(self.slice());
                    dbg!(self.span());
                    let span = self.span();
                    eprintln!("{:?}", &self.lex.source()[span.start..span.end + 10]);
                    todo!()
                }
                None => {
                    self.redent(0);
                    self.token_queue.pop_front().map(|token| Ok(token))
                },
            }
        }
    }

    pub fn span(&self) -> logos::Span {
        self.lex.span()
    }

    pub fn slice(&self) -> &str {
        self.lex.slice()
    }
}

impl<'a> Iterator for FirrtlLexer<'a> {
    type Item = Result<Token, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
