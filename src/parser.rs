use nom::{IResult};
use nom::bytes::complete::{tag};
use nom::combinator::{value, eof, opt, map};
use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::character::complete::{space0, space1, satisfy};
use nom::sequence::pair;
use nom::error::ParseError;

use crate::tokenizer::Tok;
use crate::ast::*;
use crate::{Direction, Type};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn new(msg: String) -> Error {
        Error {
            msg,
        }
    }
}

impl ParseError<&[Tok]> for Error {
    fn from_error_kind(_input: &[Tok], kind: nom::error::ErrorKind) -> Self {
        Error {
            msg: format!("{kind:?}"),
        }
    }

    fn append(_input: &[Tok], kind: nom::error::ErrorKind, other: Self) -> Self {
        Error {
            msg: format!("{}: {kind:?}", other.msg),
        }
    }
}

fn tok<'a>(expected_tok: Tok) -> impl Fn(&'a [Tok]) -> IResult<&'a [Tok], Tok, Error> {
    let run = move |input: &'a [Tok]| -> IResult<&'a [Tok], Tok, Error> {
        if input.len() == 0 {
            Err(nom::Err::Error(Error::new("Unexpected EOF".to_string())))
        } else {
            let head = &input[0];
            let tail = &input[1..];
            if head == &expected_tok {
                Ok((tail, head.clone()))
            } else {
                Err(nom::Err::Error(Error::new(format!("Unexpected token: {head:?} expected: {expected_tok:?}"))))
            }
        }
    };
    run
}

fn tok_version<'a>(input: &'a [Tok]) -> IResult<&'a [Tok], Tok, Error> {
    if input.len() == 0 {
        Err(nom::Err::Error(Error::new("Unexpected EOF".to_string())))
    } else {
        let head = &input[0];
        let tail = &input[1..];
        if let Tok::Version(_maj, _min, _pat) = head {
            Ok((tail, head.clone()))
        } else {
            Err(nom::Err::Error(Error::new(format!("Unexpected token: {head:?} (expected version)"))))
        }
    }
}

fn tok_id<'a>(input: &'a [Tok]) -> IResult<&'a [Tok], String, Error> {
    if input.len() == 0 {
        Err(nom::Err::Error(Error::new("Unexpected EOF".to_string())))
    } else {
        let head = &input[0];
        let tail = &input[1..];
        if let Tok::Ident(name) = head {
            Ok((tail, name.clone()))
        } else {
            Err(nom::Err::Error(Error::new(format!("Unexpected token: {head:?} (expected identifier)"))))
        }
    }
}

fn tok_lit<'a>(input: &'a [Tok]) -> IResult<&'a [Tok], u64, Error> {
    if input.len() == 0 {
        Err(nom::Err::Error(Error::new("Unexpected EOF".to_string())))
    } else {
        let head = &input[0];
        let tail = &input[1..];
        if let Tok::Lit(v) = head {
            Ok((tail, *v))
        } else {
            Err(nom::Err::Error(Error::new(format!("Unexpected token: {head:?} (expected lit)"))))
        }
    }
}

fn tok_info<'a>(input: &'a [Tok]) -> IResult<&'a [Tok], Tok, Error> {
    if input.len() == 0 {
        Err(nom::Err::Error(Error::new("Unexpected EOF".to_string())))
    } else {
        let head = &input[0];
        let tail = &input[1..];
        if let Tok::Info(_info) = head {
            Ok((tail, head.clone()))
        } else {
            Err(nom::Err::Error(Error::new("Unexpected token. Expected info token.".to_string())))
        }
    }
}

pub fn parse(input: &[Tok]) -> anyhow::Result<Circuit> {
    let (input, version) = tok_version(input)?;
    let (input, _) = tok(Tok::Newline)(input)?;
    let (input, _) = tok(Tok::Circuit)(input)?;
    let (input, id) = tok_id(input)?;
    dbg!(&id);
    let (input, _) = tok(Tok::Colon)(input)?;
    let (input, info) = opt(tok_info)(input)?;
    dbg!(&info);
    let (input, _) = tok(Tok::Newline)(input)?;
//    let (input, decls) = many0(alt((parse_module, parse_extmodule, parse_intmodule)))(input)?;
    let (input, _) = tok(Tok::Indent)(input)?;
    let (input, moddefs) = many0(parse_module)(input)?;
    println!("Here: {input:?}");
    let (input, _) = tok(Tok::Dedent)(input)?;
    println!("Okay?");
    let (input, _) = many0(alt((tok(Tok::Newline), tok(Tok::Dedent))))(input)?;
    if input.len() > 0 {
        println!("Leftovers: {input:?}");
    }
    //let (input, _) = eof::<&[Tok], Error>(input)?;

    Ok(Circuit {
        top: id,
        decls: moddefs.into_iter().map(|moddef| Decl::Mod(moddef)).collect(),
    })
}

fn parse_module(input: &[Tok]) -> IResult<&[Tok], ModDef, Error> {
    let (input, _) = tok(Tok::Module)(input)?;
    let (input, id) = tok_id(input)?;
    dbg!(&id);
    let (input, _) = tok(Tok::Colon)(input)?;
    let (input, info) = opt(tok_info)(input)?;
    dbg!(&info);
    let (input, _) = tok(Tok::Newline)(input)?;
    let (input, _) = tok(Tok::Indent)(input)?;
    let (input, ports) = many0(
        map(
            pair(parse_port, tok(Tok::Newline)),
            |(port, _)| port,
        )
    )(input)?;
    let (input, _) = tok(Tok::Dedent)(input)?;
    let moddef = ModDef {
        name: id.to_string(),
        ports,
        statements: vec![],
    };
    Ok((input, moddef))
}

fn parse_type(input: &[Tok]) -> IResult<&[Tok], Type, Error> {
//    let (input, constness) = opt(tok(Tok::Const))(input)?; // todo!() only for ground and aggregates
    let (input, typ) = parse_type_ground(input)?;
    Ok((input, typ))
}

fn parse_type_ground(input: &[Tok]) -> IResult<&[Tok], Type, Error> {
    if let Ok((input, typ)) = alt((
        value(Type::Clock, tok(Tok::Clock)),
        value(Type::Reset, tok(Tok::Reset)),
        value(Type::AsyncReset, tok(Tok::AsyncReset)),
    ))(input) {
        return Ok((input, typ));
    }

    let (input, tok) = alt((
            tok(Tok::UInt),
            tok(Tok::SInt),
            tok(Tok::Analog),
    ))(input)?;

    let (input, size) = opt(tok_lit)(input)?;

    let typ = match tok {
        Tok::UInt => Type::UInt(size),
        Tok::SInt => Type::SInt(size),
//        Tok::Analog => Type::Analog(size),
        _ => unreachable!(),
    };

    Ok((input, todo!()))
}

fn parse_port(input: &[Tok]) -> IResult<&[Tok], Port, Error> {
    let (input, dir) = alt((tok(Tok::Input), tok(Tok::Output)))(input)?;
    let direction = match dir {
        Tok::Input => Direction::Input,
        Tok::Output => Direction::Output,
        _ => unreachable!(),
    };

    let (input, name) = tok_id(input)?;
    let (input, _) = tok(Tok::Colon)(input)?;
    let (input, typ) = parse_type(input)?;
    let (input, info) = opt(tok_info)(input)?;
    let port = Port {
        name,
        direction,
        typ,
    };
    Ok((input, port))
}

fn parse_extmodule(input: &[Tok]) -> IResult<&[Tok], Tok, Error> { todo!() }
fn parse_intmodule(input: &[Tok]) -> IResult<&[Tok], Tok, Error> { todo!() }
