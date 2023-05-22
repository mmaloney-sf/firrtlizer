use std::io::{Read, BufReader};
pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;
use regex::Regex;

use nom::{Err, IResult, Parser};
use nom::bytes::complete::{tag};
use nom::combinator::{value, opt, eof};
use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::character::complete::{space0, satisfy};

fn parse_tokens(input: &str) -> IResult<&str, Vec<Tok>, ()> {
    let (input, tokens) = many0(parse_token)(input)?;
    eof(input)?;
    Ok(("", tokens))
}

fn parse_token(input: &str) -> IResult<&str, Tok, ()> {
    let (input, _) = space0(input)?;
    alt((
        value(Tok::Circuit, tag("circuit")),
        value(Tok::Module, tag("module")),
        value(Tok::Input, tag("input")),
        value(Tok::Output, tag("output")),
        value(Tok::Flip, tag("flip")),
        value(Tok::Colon, tag(":")),
        value(Tok::RevFatArrow, tag("<=")),
        value(Tok::LAngle, tag("<")),
        value(Tok::RAngle, tag(">")),
        value(Tok::LParen, tag("(")),
        value(Tok::RParen, tag(")")),
        value(Tok::LSquare, tag("[")),
        value(Tok::RSquare, tag("]")),
        value(Tok::LBrace, tag("{")),
        value(Tok::RBrace, tag("}")),
        value(Tok::Dot, tag(".")),
        parse_token_lit_num,
        parse_token_lit_str,
        parse_token_ident,
        parse_token_info,
    ))(input)
}

fn parse_token_info(input: &str) -> IResult<&str, Tok, ()> {
    let (input, _) = tag("@[")(input)?;
    let (input, contents) = many0(satisfy(|ch| ch != ']'))(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, Tok::Info(contents.into_iter().collect::<String>())))
}

fn parse_token_ident(input: &str) -> IResult<&str, Tok, ()> {
    let (input, head_char) = satisfy(|ch| ch.is_alphabetic())(input)?;
    let (input, tail_chars) = many0(satisfy(|ch| ch.is_alphanumeric() || ch == '_'))(input)?;
    let mut result = String::new();
    result.push(head_char);
    result.push_str(&tail_chars.into_iter().collect::<String>());
    let token = Tok::Ident(result);
    Ok((input, token))
}

fn parse_token_lit_num(input: &str) -> IResult<&str, Tok, ()> {
    let (input, number) = many1(satisfy(|ch| ch.is_numeric()))(input)?;
    let number = number.into_iter().collect::<String>();
    let token = Tok::Lit(number.parse().unwrap());
    Ok((input, token))
}

fn parse_token_lit_str(input: &str) -> IResult<&str, Tok, ()> {
    let (input, _) = tag("\"")(input)?;
    let (input, contents) = many0(satisfy(|ch| ch != '\"'))(input)?;
    let (input, _) = tag("\"")(input)?;
    let token = Tok::LitStr(contents.into_iter().collect::<String>());
    Ok((input, token))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tok {
    Unknown,
    Indent,
    Dedent,
    Newline,
    Colon,
    Input,
    Output,
    Flip,
    Ident(String),
    Lit(u64),
    LitStr(String),
    Info(String),
    Const,
    Circuit,
    Module,
    Wire,
    Reg,
    Inst,
    RevFatArrow,
    Dot,
    Mod,
    LSquare,
    RSquare,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LAngle,
    RAngle,
}

#[derive(Debug)]
pub enum LexicalError {
}

#[derive(PartialOrd, Ord, Debug, Eq, PartialEq, Clone, Default, Copy)]
pub struct Loc(usize, usize);

impl Loc {
    pub fn new(line: usize, col: usize) -> Loc {
        Loc(line, col)
    }

    pub fn line(&self) -> usize {
        self.0
    }

    pub fn col(&self) -> usize {
        self.1
    }
}

fn leading_spaces(line: &[u8]) -> usize {
    let mut i = 0;
    for ch in line {
        if *ch == ' ' as u8 {
            i += 1;
        } else {
            break;
        }
    }
    i
}

pub struct Lexer {
    loc_toks: Vec<(Loc, Tok, Loc)>,
    index: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let loc_toks: Vec<(Loc, Tok, Loc)> = vec![];

        let mut result = Lexer {
            loc_toks,
            index: 0,
        };

        for (lineno, line) in input.lines().enumerate() {
            let end_idx = match line.find(';') {
                Some(i) => i,
                None => line.len(),
            };
            let line_loc_toks = Lexer::tokenize_line(&line.as_bytes()[..end_idx], lineno, 0);
            result.loc_toks.extend_from_slice(&line_loc_toks);
        }

        result
    }

    fn tokenize_line(line: &[u8], lineno: usize, current_indent: usize) -> Vec<(Loc, Tok, Loc)> {
        let mut toks = vec![];
        let mut line = String::from_utf8(line.to_vec()).unwrap();
        println!("{line}");
        while line != "" {
            let (line2, tok) = parse_token(&line).unwrap();
            println!("{tok:?}");
            toks.push(tok);
            line = line2.to_string();
        }

        toks.into_iter().map(|tok| (Loc::default(), tok, Loc::default())).collect()
        /*
        match parse_tokens(&line) {
            Ok((_, toks)) => return toks.into_iter().map(|tok| (Loc::default(), tok, Loc::default())).collect(),
            Err(e) => { eprintln!("{e:?}"); panic!() },
        }
        */
    }
        /*
        let mut loc_toks = vec![];
        let leading_spaces = leading_spaces(line);
        let loc = Loc::new(lineno, 0);
        if leading_spaces > current_indent {
            loc_toks.push((loc.clone(), Tok::Indent, loc));
        } else if leading_spaces < current_indent {
            loc_toks.push((loc.clone(), Tok::Indent, loc));
        }

        let re_flip = Regex::new(r"flip( (.*)|$)").unwrap();
        let re_circuit = Regex::new(r"circuit( (.*)|$)").unwrap();
        let mut col = 0;

        while col < line.len() {
            let start_loc = Loc::new(lineno, col);
            if re_flip.is_match(&line[col..]) {
                loc_toks.push((start_loc, Tok::Flip, start_loc));
            } else if re_circuit.is_match(&line[col..]) {
                loc_toks.push((start_loc, Tok::Circuit, start_loc));
            } else {
                loc_toks.push((start_loc, Tok::Unknown, start_loc));
            }
            col += 1;
        }
        let loc = Loc::new(lineno, line.len());
        loc_toks.push((loc.clone(), Tok::Newline, loc));
        loc_toks
        */
}

impl Iterator for Lexer {
    type Item = Spanned<Tok, Loc, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.loc_toks.len() {
            let loc_tok = self.loc_toks[self.index].clone();
            self.index += 1;
            Some(Ok(loc_tok))
        } else {
            None
        }
    }
}
