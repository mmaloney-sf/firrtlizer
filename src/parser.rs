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
pub struct ParseErr {
    msg: String,
}

impl ParseErr {
    pub fn new(msg: String) -> ParseErr {
        ParseErr {
            msg,
        }
    }
}

impl<'a> ParseError<&[Tok<'a>]> for ParseErr {
    fn from_error_kind(_input: &[Tok<'a>], kind: nom::error::ErrorKind) -> Self {
        ParseErr {
            msg: format!("{kind:?}"),
        }
    }

    fn append(_input: &[Tok<'a>], kind: nom::error::ErrorKind, other: Self) -> Self {
        ParseErr {
            msg: format!("{}: {kind:?}", other.msg),
        }
    }
}

fn tok<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], &'b Tok<'a>, ParseErr> {
    if input.len() > 0 {
        let tok = &input[0];
        let input = &input[1..];
        Ok((input, tok))
    } else {
        Err(nom::Err::Error(ParseErr::new(format!("Unexpected EOF"))))
    }
}

fn consume_keyword<'a: 'b, 'b>(keyword: &'static str) -> impl Fn(&'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], &'a str, ParseErr> + '_ {
    move |input| {
        let (input, tok) = tok(input)?;
        if let Tok::Id(keyword0) = tok {
            if keyword == *keyword0 {
                return Ok((input, keyword));
            }
        }
        Err(nom::Err::Error(ParseErr::new(format!("Unexpected token: {tok:?} expected: {keyword:?}"))))
    }
}

fn consume_lit<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], u64, ParseErr> {
    let (input, tok) = tok(input)?;
    if let Tok::Lit(v) = tok {
        return Ok((input, *v));
    }
    Err(nom::Err::Error(ParseErr::new(format!("Bad thing"))))
}

fn consume_id<'a, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], &'a str, ParseErr> {
    if let Ok((input, Tok::Id(id))) = tok(input) {
        Ok((input, id))
    } else {
        Err(nom::Err::Failure(ParseErr::new(format!("Expected identifier"))))
    }
}

fn consume_punc<'a: 'b, 'b>(punc: &str) -> impl Fn(&'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], (), ParseErr> + '_ {
    move |input| {
        let (input, tok) = tok(input)?;
        if let Tok::Punc(punc0) = tok {
            if punc == *punc0 {
                Ok((input, ()))
            } else {
                Err(nom::Err::Error(ParseErr::new(format!("Unexpected token: {punc0:?}. expected: {punc:?}"))))
            }
        } else {
            Err(nom::Err::Error(ParseErr::new(format!("Unexpected EOF"))))
        }
    }
}

fn consume_newline<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], (), ParseErr> {
    if let Ok((input, Tok::Newline)) = tok(input) {
        Ok((input, ()))
    } else {
        Err(nom::Err::Failure(ParseErr::new(format!("Expected newline"))))
    }
}

fn tok_version<'a, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], (usize, usize, usize), ParseErr> {
    let (input, Tok::Version(maj, min, pat)) = tok(input)? else {
        return Err(nom::Err::Failure(ParseErr::new(format!("Expected version"))));
    };
    Ok((input, (*maj, *min, *pat)))
}

fn try_consume_info<'a, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Option<&'a str>, ParseErr> {
    let (input, Tok::Info(info)) = tok(input)? else {
        return Ok((input, None));
    };
    Ok((input, Some(info)))
}

fn consume_indent<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], (), ParseErr> {
    let (input, tok) = tok(input)?;
    if let Tok::Indent(_, _amount) = tok {
        Ok((input, ()))
    } else {
        Err(nom::Err::Failure(ParseErr::new(format!("Expected indent, but found {tok:?}"))))
    }
}

fn consume_dedent<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], (), ParseErr> {
    let (input, tok) = tok(input)?;
    if let Tok::Dedent(_, _amount) = tok {
        Ok((input, ()))
    } else {
        Err(nom::Err::Failure(ParseErr::new(format!("Expected dedent, but found {tok:?}"))))
    }
}

/*
fn flip_tok(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Flippedness, Error> {
    let (input, ok) = opt(expect_tok(Tok::Flip))(input)?;
    match ok {
        Some(()) => Ok((input, Flippedness::Flipped)),
        None => Ok((input, Flippedness::Aligned)),
    }
}

fn parse_type_aggregate(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Type, Error> {
    alt((
        parse_type_aggregate_bundle,
        parse_type_aggregate_vec,
    ))(input)
}

fn parse_type_aggregate_bundle(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Type, Error> {
    let (input, _) = expect_tok(Tok::LBrace)(input)?;
    let (input, fields) = many1(parse_field)(input)?;
    let (input, _) = expect_tok(Tok::RBrace)(input)?;
    Ok((input, Type::Bundle(fields)))
}

fn parse_type_aggregate_vec(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Type, Error> {
    Err(nom::Err::Error(Error::new(format!("Not implemented: parse_type_aggregate_vec"))))
    // | type , "[" , int_any , "]" ;
}

fn parse_field(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], BundleField, Error> {
    let (input, flip) = flip_tok(input)?;
    let (input, name) = tok_id(input)?;
    let (input, _) = expect_tok(Tok::Colon)(input)?;
    let (input, typ) = parse_type(input)?;
    let field = BundleField(flip, name, Box::new(typ));
    Ok((input, field))
}


fn parse_extmodule(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Tok, Error> { todo!() }
fn parse_intmodule(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Tok, Error> { todo!() }
*/

fn parse_type<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Type, ParseErr> {
//    let (input, constness) = opt(tok(Tok::Const))(input)?; // todo!() only for ground and aggregates
    let (input, typ) = alt((
        parse_type_ground,
//        parse_type_aggregate,
    ))(input)?;
    Ok((input, typ))
}

fn parse_type_ground<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Type, ParseErr> {
    if let Ok((input, typ)) = alt((
        value(Type::Clock, consume_keyword("Clock")),
        value(Type::Reset, consume_keyword("Reset")),
        value(Type::AsyncReset, consume_keyword("AsyncReset")),
    ))(input) {
        return Ok((input, typ));
    }

    let (input, typ_name) = alt((
            consume_keyword("UInt"),
            consume_keyword("SInt"),
//            consume_keyword("Analog"),
    ))(input)?;

    let (input, width) = opt(parse_width)(input)?;

    let typ = match typ_name {
        "UInt" => Type::UInt(width),
        "SInt" => Type::SInt(width),
        //"Analog" => Type::Analog(width),
        _ => unreachable!(),
    };

    Ok((input, typ))
}

fn parse_width<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], u64, ParseErr> {
    let (input, _) = consume_punc("<")(input)?;
    let (input, v) = consume_lit(input)?;
    let (input, _) = consume_punc(">")(input)?;
    Ok((input, v))
}

fn parse_direction<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Direction, ParseErr> {
    alt((
        value(Direction::Input, consume_keyword("input")),
        value(Direction::Output, consume_keyword("output")),
    ))(input)
}

fn parse_port<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Port, ParseErr> {
    let (input, direction) = parse_direction(input)?;

    let Ok((input, name)) = consume_id(input) else {
        return Err(nom::Err::Failure(ParseErr::new(format!("Expected id for port"))));
    };
    let name = name.to_string();
    let (input, _) = consume_punc(":")(input)?;
    let (input, typ) = parse_type(input)?;
    let (input, info) = try_consume_info(input)?;
    let port = Port {
        name,
        direction,
        typ,
    };
    Ok((input, port))
}

fn parse_module<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], ModDef, ParseErr> {
    let (input, _) = consume_keyword("module")(input)?;
    let (input, id) = consume_id(input)?;
    let (input, _) = consume_punc(":")(input)?;
    let (input, info) = try_consume_info(input)?;
    let (input, _) = consume_newline(input)?;
    let (input, _) = consume_indent(input)?;

    let Ok((input, ports)) = many0(
        map(
            pair(parse_port, consume_newline),
            |(port, _)| port,
        )
    )(input) else {
        return Err(nom::Err::Failure(ParseErr::new(format!("Expected ports"))));
    };

    let (input, _) = consume_dedent(input)?;

    let moddef = ModDef {
        name: id.to_string(),
        ports,
        statements: vec![],
    };
    Ok((input, moddef))
}

fn parse_decl<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Decl, ParseErr> {
    alt((
        map(parse_module, |m| Decl::Mod(m)),
    ))(input)
}

pub fn parse<'a>(input: &[Tok<'a>]) -> anyhow::Result<Circuit> {
    let (input, version) = tok_version(input)?;
    let (input, _) = consume_newline(input)?;
    let (input, _) = consume_keyword("circuit")(input)?;
    let (input, top) = consume_id(input)?;
    let top = top.to_string();
    let (input, _) = consume_punc(":")(input)?;

    let (input, info) = try_consume_info(input)?;

    let (input, _) = consume_newline(input)?;
    let (input, _) = consume_indent(input)?;
    let (input, decls) = many1(parse_decl)(input)?;
    let (input, _) = consume_dedent(input)?;

    Ok(Circuit {
        top,
        decls,
    })
}
