use crate::ast::*;

use nom::{Err, IResult};
use nom::bytes::complete::{tag};
use nom::combinator::{value, opt, eof};
use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::character::complete::{space0, satisfy};
use nom::sequence::pair;

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
    EqEq,
    Eq,
    Ident(String),
    Lit(u64),
    LitStr(String),
    Info(String),
    Const,
    Circuit,
    Module,
    Wire,
    Node,
    Reg,
    Mem,
    Inst,
    RevFatArrow,
    Dot,
    Comma,
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

pub fn tokenize(input: &str) -> anyhow::Result<Vec<Tok>> {
    let mut indent_level: isize = 0;
    let spaces_per_indent_level: isize = 2;

    let mut toks = vec![];
    for line in input.lines() {
        let leading_spaces = leading_spaces(line.as_bytes()) as isize;
        if leading_spaces > indent_level * spaces_per_indent_level {
            toks.push(Tok::Indent);
            indent_level += 1;
        } else if leading_spaces < indent_level * spaces_per_indent_level {
            toks.push(Tok::Dedent);
            indent_level -= 1;
        }
        println!("{line}");
        let line = String::from_utf8((&line.as_bytes()[leading_spaces as usize..]).to_vec()).unwrap();
        let (_, line_toks) = tokenize_line(&line)?;
        for tok in line_toks {
            toks.push(tok);
        }

        toks.push(Tok::Newline);
    }
    Ok(toks)
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

fn tokenize_line(input: &str) -> IResult<&str, Vec<Tok>, ()> {
    let mut rest = input;
    let mut toks = vec![];
    loop {
        let end = alt((eof::<&str, ()>, tag(";")))(rest);
        if let Ok((input, _)) = end {
            return Ok((input, toks));
        }

        let (input, tok) = parse_token(rest)?;
        toks.push(tok);
        rest = input;
    }
}

fn parse_token(input: &str) -> IResult<&str, Tok, ()> {
    let (input, tok) = alt((
        parse_keyword,
        value(Tok::Colon, tag(":")),
        value(Tok::EqEq, tag("==")),
        value(Tok::Eq, tag("=")),
        value(Tok::RevFatArrow, tag("<=")),
        value(Tok::Dot, tag(".")),
        value(Tok::Comma, tag(",")),
        parse_token_lp,
        parse_token_lit_num,
        parse_token_lit_str,
        parse_token_ident,
        parse_token_info,
    ))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, tok))
}

fn parse_keyword(input: &str) -> IResult<&str, Tok, ()> {
    alt((
        value(Tok::Circuit, tag("circuit")),
        value(Tok::Module, tag("module")),
        value(Tok::Input, tag("input")),
        value(Tok::Output, tag("output")),
        value(Tok::Wire, tag("wire")),
        value(Tok::Node, tag("node")),
        value(Tok::Reg, tag("reg")),
        value(Tok::Mem, tag("mem")),
        value(Tok::Flip, tag("flip")),
    ))(input)
}

fn parse_token_lp(input: &str) -> IResult<&str, Tok, ()> {
    alt((
        value(Tok::LAngle, tag("<")),
        value(Tok::RAngle, tag(">")),
        value(Tok::LParen, tag("(")),
        value(Tok::RParen, tag(")")),
        value(Tok::LSquare, tag("[")),
        value(Tok::RSquare, tag("]")),
        value(Tok::LBrace, tag("{")),
        value(Tok::RBrace, tag("}")),
    ))(input)
}

fn parse_token_info(input: &str) -> IResult<&str, Tok, ()> {
    let (input, _) = tag("@[")(input)?;
    let (input, contents) = many0(satisfy(|ch| ch != ']'))(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, Tok::Info(contents.into_iter().collect::<String>())))
}

fn parse_token_ident(input: &str) -> IResult<&str, Tok, ()> {
    let (input, head_char) = satisfy(|ch| ch.is_alphabetic() || ch == '_')(input)?;
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
    let (input, contents) = many0(parse_token_lit_content_char)(input)?;
    let (input, _) = tag("\"")(input)?;
    let token = Tok::LitStr(contents.into_iter().collect::<String>());
    Ok((input, token))
}

fn parse_token_lit_content_char(input: &str) -> IResult<&str, char, ()> {
    alt((
        satisfy::<_, &str, ()>(|ch| ch != '\"' && ch != '\\'),
        parse_token_lit_content_char_esc,
    ))(input)
}

fn parse_token_lit_content_char_esc(input: &str) -> IResult<&str, char, ()> {
    let (input, (_v1, v2)) = pair(
        satisfy(|ch| ch == '\\'),
        alt((
            value('\\', tag("\\")),
            value('"', tag("\"")),
            value('\n', tag("n")),
            value('\t', tag("t")),
//                value('', tag("]")), // todo!() really?
        ))
    )(input)?;
    Ok((input, v2))
}
