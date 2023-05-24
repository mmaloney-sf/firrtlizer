use nom::IResult;
use nom::bytes::complete::tag;
use nom::combinator::{value, eof, opt, map};
use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::character::complete::{space0, space1, satisfy};
use nom::sequence::pair;
use nom::error::ParseError;

use crate::tokenizer::Tok;
use crate::ast::*;
use crate::{Direction, Type, BundleField, Flippedness};

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

fn expect_tok<'a>(expected_tok: Tok) -> impl Fn(&'a [Tok]) -> IResult<&'a [Tok], (), Error> {
    let run = move |input: &'a [Tok]| -> IResult<&'a [Tok], (), Error> {
        let (input, tok) = tok(input)?;
        if tok != expected_tok {
            Err(nom::Err::Error(Error::new(format!("Unexpected token. expected: {expected_tok:?}"))))
        } else {
            Ok((input, ()))
        }
    };
    run
}

fn tok(input: &[Tok]) -> IResult<&[Tok], Tok, Error> {
    if input.len() == 0 {
        Err(nom::Err::Error(Error::new("Unexpected EOF".to_string())))
    } else {
        let head = &input[0];
        let tail = &input[1..];
        Ok((tail, head.clone()))
    }
}

fn dedent_tok(input: &[Tok]) -> IResult<&[Tok], (), Error> {
    let (input, tok) = tok(input)?;
    if let Tok::Dedent(_amount) = tok {
        Ok((input, ()))
    } else {
        Err(nom::Err::Failure(Error::new(format!("Unexpected token: {tok:?} expected: Dedent"))))
    }
}

fn indent_tok(input: &[Tok]) -> IResult<&[Tok], (), Error> {
    let (input, tok) = tok(input)?;
    if let Tok::Indent(_amount) = tok {
        Ok((input, ()))
    } else {
        Err(nom::Err::Failure(Error::new(format!("Unexpected token: {tok:?} expected: Indent"))))
    }
}

fn dir_tok(input: &[Tok]) -> IResult<&[Tok], Direction, Error> {
    let (input, tok) = tok(input)?;
    if let Tok::Input = tok {
        Ok((input, Direction::Input))
    } else if let Tok::Output = tok {
        Ok((input,  Direction::Output))
    } else {
        Err(nom::Err::Error(Error::new(format!("Unexpected token: {tok:?} expected: Indent"))))
    }
}

fn flip_tok(input: &[Tok]) -> IResult<&[Tok], Flippedness, Error> {
    let (input, ok) = opt(expect_tok(Tok::Flip))(input)?;
    match ok {
        Some(()) => Ok((input, Flippedness::Flipped)),
        None => Ok((input, Flippedness::Aligned)),
    }
}

/*
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
*/

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
    let (input, _) = expect_tok(Tok::Newline)(input)?;
    let (input, _) = expect_tok(Tok::Circuit)(input)?;
    let (input, id) = tok_id(input)?;
    dbg!(&id);
    let (input, _) = expect_tok(Tok::Colon)(input)?;
    let (input, info) = opt(tok_info)(input)?;
    dbg!(&info);
    let (input, _) = expect_tok(Tok::Newline)(input)?;
//    let (input, decls) = many0(alt((parse_module, parse_extmodule, parse_intmodule)))(input)?;
    println!("input: {input:?}");
    let (input, _) = indent_tok(input)?;
    println!("input: {input:?}");
    let (input, moddefs) = many0(parse_module)(input)?;
    println!("Here: {input:?}");
    let (input, _) = dedent_tok(input)?;
    println!("Okay?");
    let (input, _) = many0(alt((expect_tok(Tok::Newline), dedent_tok)))(input)?;
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
    let (input, _) = expect_tok(Tok::Module)(input)?;
    let (input, id) = tok_id(input)?;
    dbg!(&id);
    let (input, _) = expect_tok(Tok::Colon)(input)?;
    let (input, info) = opt(tok_info)(input)?;
    dbg!(&info);
    let (input, _) = expect_tok(Tok::Newline)(input)?;
    let (input, _) = indent_tok(input)?;
    let Ok((input, ports)) = many0(
        map(
            pair(parse_port, expect_tok(Tok::Newline)),
            |(port, _)| port,
        )
    )(input) else {
        return Err(nom::Err::Failure(Error::new(format!("Expected ports"))));
    };

    println!("toks: {input:?}");
    let (input, _) = dedent_tok(input)?;
    println!("toks: {input:?}");
    let moddef = ModDef {
        name: id.to_string(),
        ports,
        statements: vec![],
    };
    Ok((input, moddef))
}

fn parse_type(input: &[Tok]) -> IResult<&[Tok], Type, Error> {
//    let (input, constness) = opt(tok(Tok::Const))(input)?; // todo!() only for ground and aggregates
    let (input, typ) = alt((
        parse_type_ground,
        parse_type_aggregate,
    ))(input)?;
    Ok((input, typ))
}

fn parse_type_ground(input: &[Tok]) -> IResult<&[Tok], Type, Error> {
    if let Ok((input, typ)) = alt((
        value(Type::Clock, expect_tok(Tok::Clock)),
        value(Type::Reset, expect_tok(Tok::Reset)),
        value(Type::AsyncReset, expect_tok(Tok::AsyncReset)),
    ))(input) {
        return Ok((input, typ));
    }

    let (input, tok) = alt((
            value(Tok::UInt, expect_tok(Tok::UInt)),
            value(Tok::SInt, expect_tok(Tok::SInt)),
            value(Tok::Analog, expect_tok(Tok::Analog)),
    ))(input)?;

    let (input, size) = opt(tok_lit)(input)?;

    let typ = match tok {
        Tok::UInt => Type::UInt(size),
        Tok::SInt => Type::SInt(size),
//        Tok::Analog => Type::Analog(size),
        _ => unreachable!(),
    };

    Ok((input, typ))
}

fn parse_type_aggregate(input: &[Tok]) -> IResult<&[Tok], Type, Error> {
    alt((
        parse_type_aggregate_bundle,
        parse_type_aggregate_vec,
    ))(input)
}

fn parse_type_aggregate_bundle(input: &[Tok]) -> IResult<&[Tok], Type, Error> {
    let (input, _) = expect_tok(Tok::LBrace)(input)?;
    let (input, fields) = many1(parse_field)(input)?;
    let (input, _) = expect_tok(Tok::RBrace)(input)?;
    Ok((input, Type::Bundle(fields)))
}

fn parse_type_aggregate_vec(input: &[Tok]) -> IResult<&[Tok], Type, Error> {
    Err(nom::Err::Error(Error::new(format!("Not implemented: parse_type_aggregate_vec"))))
    // | type , "[" , int_any , "]" ;
}

fn parse_field(input: &[Tok]) -> IResult<&[Tok], BundleField, Error> {
    let (input, flip) = flip_tok(input)?;
    let (input, name) = tok_id(input)?;
    let (input, _) = expect_tok(Tok::Colon)(input)?;
    let (input, typ) = parse_type(input)?;
    let field = BundleField(flip, name, Box::new(typ));
    Ok((input, field))
}

fn parse_port(input: &[Tok]) -> IResult<&[Tok], Port, Error> {
    let (input, direction) = dir_tok(input)?;

    let (input, name) = tok_id(input)?;
    let (input, _) = expect_tok(Tok::Colon)(input)?;
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
