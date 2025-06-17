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
    #[token(r"flip")]
    KwFlip,
    #[token(r"input")]
    KwInput,
    #[token(r"output")]
    KwOutput,
    #[token(r"UInt")]
    KwUInt,
    #[token(r"SInt")]
    KwSInt,
    #[token(r"Clock")]
    KwClock,
    #[token(r"Reset")]
    KwReset,
    #[token(r"Probe")]
    KwProbe2,
    #[token(r"RwProbe")]
    KwRwProbe2,
    #[token(r"wire")]
    KwWire,
    #[token(r"cmem")]
    KwCmem,
    #[token(r"reg")]
    KwReg,
    #[token(r"regreset")]
    KwRegReset,
    #[token(r"node")]
    KwNode,
    #[token(r"is")]
    KwIs,
    #[token(r"invalid")]
    KwInvalid,
    #[token(r"invalidate")]
    KwInvalidate,
    #[token(r"asClock")]
    KwAsClock,
    #[token(r"asAsyncReset")]
    KwAsAsyncReset,
    #[token(r"inst")]
    KwInst,
    #[token(r"connect")]
    KwConnect,
    #[token(r"of")]
    KwOf,
    #[token(r"extmodule")]
    KwExtModule,
    #[token(r"defname")]
    KwDefName,
    #[token(r"data-type")]
    KwDataType,
    #[token(r"read-latency")]
    KwReadLatency,
    #[token(r"write-latency")]
    KwWriteLatency,
    #[token(r"layer")]
    KwLayer,
    #[token(r"layerblock")]
    KwLayerBlock,
    #[token(r"bits")]
    KwBits,
    #[token(r"when")]
    KwWhen,
    #[token(r"else")]
    KwElse,
    #[token(r"asUInt")]
    KwasUInt,
    #[token(r"asSInt")]
    KwAsSInt,
    #[token(r"cvt")]
    KwCvt,
    #[token(r"neg")]
    KwNeg,
    #[token(r"not")]
    KwNot,
    #[token(r"andr")]
    KwAndr,
    #[token(r"orr")]
    KwOrr,
    #[token(r"xorr")]
    KwXorr,
    #[token(r"add")]
    KwAdd,
    #[token(r"sub")]
    KwSub,
    #[token(r"mul")]
    KwMul,
    #[token(r"div")]
    KwDiv,
    #[token(r"rem")]
    KwRem,
    #[token(r"lt")]
    KwLt,
    #[token(r"leq")]
    KwLeq,
    #[token(r"gt")]
    KwGt,
    #[token(r"geq")]
    KwGeq,
    #[token(r"eq")]
    KwEq,
    #[token(r"neq")]
    KwNeq,
    #[token(r"dshl")]
    KwDshl,
    #[token(r"dshr")]
    KwDshr,
    #[token(r"and")]
    KwAnd,
    #[token(r"or")]
    KwOr,
    #[token(r"xor")]
    KwXor,
    #[token(r"cat")]
    KwCat,
    #[token(r"intrinsic")]
    KwIntrinsic,
    #[token(r"pad")]
    KwPad,
    #[token(r"shl")]
    KwShl,
    #[token(r"shr")]
    KwShr,
    #[token(r"head")]
    KwHead,
    #[token(r"tail")]
    KwTail,
    #[token(r"mux")]
    KwMux,
    #[token(r"read")]
    KwRead,
    #[token(r"infer")]
    KwInfer,
    #[token(r"mport")]
    KwMport,
    #[token(r"define")]
    KwDefine,
    #[token(r"probe")]
    KwProbe,
    #[token(r"rwprobe")]
    KwRwProbe,

    #[regex(r"([a-zA-Z_][a-zA-Z_0-9]*|`[^`]+`)")]
    Id,
    #[regex(r"-?([0-9]+|0h[0-9a-fA-F]+)")]
    Int,

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
    #[token(r"<=")]
    RevFatArrow,
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
                    eprintln!();
                    panic!()
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
