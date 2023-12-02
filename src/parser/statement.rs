use super::*;
use super::expr::parse_expr;
use crate::{RefPath};
use crate::ast::Statement;
use nom::IResult;

pub fn parse_statement<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    alt((
        parse_circuit_component,
        parse_connectlike,
        value(Statement::Skip, consume_keyword("skip")),
        parse_statement_wire,
        parse_statement_connect_old,
    ))(input)
    //let (input, info) = try_consume_info(input)?;
}

fn parse_circuit_component<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    alt((
        parse_circuit_component_node,
    ))(input)
}

fn parse_circuit_component_node<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    let (input, _) = consume_keyword("node")(input)?;
    let (input, name) = consume_id(input)?;
    let (input, _) = consume_punc("=")(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((input, Statement::Node(name.to_string(), Box::new(expr))))
}

fn parse_statement_wire<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    let (input, _) = consume_keyword("wire")(input)?;
    let (input, name) = consume_id(input)?;
    let (input, _) = consume_punc(":")(input)?;
    let (input, typ) = parse_type(input)?;
    let (input, _info) = try_consume_info(input)?;
    Ok((input, Statement::Wire(name.to_string(), typ)))
}

fn parse_statement_connect_old<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    let (input, refpath) = parse_reference(input)?;
    let (input, _) = consume_punc("<=")(input)?;
    let (input, expr) = parse_expr(input)?;
    let (input, _info) = try_consume_info(input)?;
    Ok((input, Statement::Connect(refpath, Box::new(expr))))
}

fn parse_connectlike<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    alt((
        parse_connect,
        parse_invalidate,
    ))(input)
}

fn parse_connect<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    let (input, _) = consume_keyword("connect")(input)?;
    let (input, r) = parse_reference(input)?;
    let (input, _) = consume_punc(",")(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((input, Statement::Connect(r, Box::new(expr))))
}

fn parse_invalidate<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    let (input, _) = consume_keyword("invalidate")(input)?;
    let (input, r) = parse_reference(input)?;
    Ok((input, Statement::Invalidate(r)))
}
