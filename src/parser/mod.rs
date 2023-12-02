pub mod statement;
pub mod expr;

use nom::IResult;
use nom::combinator::{value, eof, opt, map};
use nom::branch::alt;
use nom::multi::{many0, many1, separated_list0};
use nom::sequence::pair;
use nom::error::ParseError;

use crate::tokenizer::Tok;
use crate::RefPath;
use crate::ast::*;
use crate::{Direction, Type, BundleField, Flippedness};
pub use statement::parse_statement;

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
        Err(nom::Err::Error(ParseErr::new(format!("Expected identifier"))))
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
        Err(nom::Err::Error(ParseErr::new(format!("Expected newline"))))
    }
}

fn consume_newlines<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], (), ParseErr> {
    let (input, _) = many1(consume_newline)(input)?;
    Ok((input, ()))
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
    if let Tok::Indent(__, _amount) = tok {
        Ok((input, ()))
    } else {
        Err(nom::Err::Failure(ParseErr::new(format!("Expected indent, but found {tok:?}"))))
    }
}

fn consume_dedent<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], (), ParseErr> {
    let (input, tok) = tok(input)?;
    if let Tok::Dedent(__, _amount) = tok {
        Ok((input, ()))
    } else {
        Err(nom::Err::Failure(ParseErr::new(format!("Expected dedent, but found {tok:?}"))))
    }
}

/*

fn parse_type_aggregate_vec(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Type, Error> {
    Err(nom::Err::Error(Error::new(format!("Not implemented: parse_type_aggregate_vec"))))
    // | type , "[" , int_any , "]" ;
}

fn parse_extmodule(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Tok, Error> { todo!() }
fn parse_intmodule(input: &[Tok<'a>]) -> IResult<&[Tok<'a>], Tok, Error> { todo!() }
*/

fn parse_reference<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], RefPath, ParseErr> {
    let (input, refpath) = alt((
        parse_reference_static,
        parse_reference_dynamic,
    ))(input)?;
    Ok((input, refpath))
}

fn parse_reference_static<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], RefPath, ParseErr> {
    let (mut input, name) = consume_id(input)?;
    let mut refpath = RefPath::Id(name.to_string());

    loop {
        if let Some(Tok::Punc(".")) = input.get(0) {
            let (rest, _) = consume_punc(".")(input)?;
            let (rest, name) = consume_id(rest)?;
            input = rest;
            refpath = RefPath::Dot(Box::new(refpath), name.to_string());
        } else if let Some(Tok::Punc("[")) = input.get(0) {
            let (rest, _) = consume_punc("[")(input)?;
            let (rest, idx) = consume_lit(rest)?;
            let (rest, _) = consume_punc("]")(rest)?;
            input = rest;
            refpath = RefPath::Index(Box::new(refpath), idx as isize);
        } else {
            break;
        }

    }
//    parse_vec_size
    Ok((input, refpath))
}

fn parse_reference_dynamic<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], RefPath, ParseErr> {
    let (input, name) = consume_id(input)?;
    let mut refpath = RefPath::Id(name.to_string());
//    parse_vec_size
    Ok((input, refpath))
}

fn parse_type<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Type, ParseErr> {
//    let (input, constness) = opt(tok(Tok::Const))(input)?; // todo!() only for ground and aggregates
    let (input, mut typ) = alt((
        parse_type_ground,
        parse_type_bundle,
    ))(input)?;

    let (input, vec_sizes) = many0(parse_vec_size)(input)?;
    for vec_size in vec_sizes {
        typ = Type::Vec(vec_size as usize, Box::new(typ));
    }

    Ok((input, typ))
}

#[test]
fn test_parse_type() {
    let typ = "{ clock : Clock, reset : Reset}";
    let toks: Vec<Tok> = crate::tokenizer::tokenize(typ).unwrap();
    let toks = &toks[..toks.len()-1];
    parse_type(toks).unwrap();
}

fn parse_vec_size<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], u64, ParseErr> {
    let (input, _) = consume_punc("[")(input)?;
    let (input, v) = consume_lit(input)?;
    let (input, _) = consume_punc("]")(input)?;
    Ok((input, v))
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

fn parse_type_bundle<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Type, ParseErr> {
    let (input, _) = consume_punc("{")(input)?;
    let (input, fields) = separated_list0(consume_punc(","), parse_field)(input)?;
    let (input, _) = consume_punc("}")(input)?;
    let typ = Type::Bundle(fields);
    Ok((input, typ))
}

fn parse_flip<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Flippedness, ParseErr> {
    let (input, ok) = opt(consume_keyword("flip"))(input)?;
    match ok {
        Some(_) => Ok((input, Flippedness::Flipped)),
        None => Ok((input, Flippedness::Aligned)),
    }
}

fn parse_field<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], BundleField, ParseErr> {
    let (input, flip) = parse_flip(input)?;
    let (input, name) = consume_id(input)?;
    let (input, _) = consume_punc(":")(input)?;
    let (input, typ) = parse_type(input)?;
    let field = BundleField(flip, name.to_string(), Box::new(typ));
    Ok((input, field))
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

    let (input, ports) = match many0(
        map(
            pair(parse_port, consume_newlines),
            |(port, _)| port,
        )
    )(input) {
        Ok((input, ports)) => (input, ports),
        Err(e) => return Err(e),
    };

    let (input, statements) = match many0(
        map(
            pair(parse_statement, consume_newlines),
            |(stmt, _)| stmt,
        )
    )(input) {
        Ok((input, ports)) => (input, ports),
        Err(e) => return Err(e),
    };

    let (input, statements) = match many0(
        map(
            pair(parse_statement, consume_newlines),
            |(stmt, _)| stmt,
        )
    )(input) {
        Ok((input, ports)) => (input, ports),
        Err(e) => return Err(e),
    };

//    let (input, _) = consume_dedent(input)?;

    let moddef = ModDef {
        name: id.to_string(),
        ports,
        statements,
    };
    Ok((input, moddef))
}

pub fn parse_decl<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Decl, ParseErr> {
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

    let (input, _) = many0(consume_newline)(input)?;
    let (_input, _) = consume_dedent(input)?;

    Ok(Circuit {
        top,
        decls,
    })
}
