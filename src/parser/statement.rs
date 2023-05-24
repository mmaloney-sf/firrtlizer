use super::*;
use super::expr::parse_expr;
use crate::{RefPath};
use crate::ast::Statement;
use nom::IResult;

pub(crate) fn parse_statement<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    alt((
        value(Statement::Skip, consume_keyword("skip")),
        parse_statement_wire,
        parse_statement_connect_old,
    ))(input)
}

fn parse_statement_wire<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    let (input, _) = consume_keyword("wire")(input)?;
    let (input, name) = consume_id(input)?;
    let (input, _) = consume_punc(":")(input)?;
    let (input, typ) = parse_type(input)?;
    let (input, info) = try_consume_info(input)?;
    Ok((input, Statement::Wire(name.to_string(), typ)))
}

fn parse_statement_connect_old<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    let (input, refpath) = parse_reference(input)?;
    println!("toks: {input:?}");
    let (input, _) = consume_punc("<=")(input)?;
    let (input, expr) = parse_expr(input)?;
    let (input, info) = try_consume_info(input)?;
    Ok((input, Statement::Connect(refpath, Box::new(expr))))
}
