use super::*;
use super::expr::parse_expr;
use crate::ast::Statement;
use nom::IResult;
use nom::combinator::map;

pub fn parse_statement<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    alt((
        parse_circuit_component,
        parse_connectlike,
        value(Statement::Skip, consume_keyword("skip")),
    ))(input)
    //let (input, info) = try_consume_info(input)?;
}

fn parse_circuit_component<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    let (input, circuit_component) = alt((
        parse_circuit_component_node,
        parse_circuit_component_reg,
    ))(input)?;
    Ok((input, Statement::CircuitComponent(circuit_component)))
}

fn parse_circuit_component_node<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], CircuitComponent, ParseErr> {
    let (input, _) = consume_keyword("node")(input)?;
    let (input, name) = consume_id(input)?;
    let (input, _) = consume_punc("=")(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((input, CircuitComponent::Node(name.to_string(), Box::new(expr))))
}

fn parse_circuit_component_reg<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], CircuitComponent, ParseErr> {
    alt((
        parse_circuit_component_reg_reg,
        parse_circuit_component_reg_regreset,
    ))(input)
}

fn parse_circuit_component_reg_reg<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], CircuitComponent, ParseErr> {
    let (input, _) = consume_keyword("reg")(input)?;
    let (input, name) = consume_id(input)?;
    let (input, _) = consume_punc(":")(input)?;
    let (input, typ) = parse_type(input)?;
    let (input, _) = consume_punc(",")(input)?;
    let (input, clock) = parse_expr(input)?;
    Ok((input, CircuitComponent::Node(name.to_string(), Box::new(clock)))) // TODO
}

fn parse_circuit_component_reg_regreset<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], CircuitComponent, ParseErr> {
    let (input, _) = consume_keyword("regreset")(input)?;
    let (input, name) = consume_id(input)?;
    let (input, _) = consume_punc(":")(input)?;
    let (input, typ) = parse_type(input)?;
    let (input, _) = consume_punc(",")(input)?;
    let (input, clock) = parse_expr(input)?;
    let (input, _) = consume_punc(",")(input)?;
    let (input, reset) = parse_expr(input)?;
    let (input, _) = consume_punc(",")(input)?;
    let (input, reset_val) = parse_expr(input)?;
    Ok((input, CircuitComponent::Node(name.to_string(), Box::new(clock)))) // TODO
}

fn parse_statement_wire<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], CircuitComponent, ParseErr> {
    let (input, _) = consume_keyword("wire")(input)?;
    let (input, name) = consume_id(input)?;
    let (input, _) = consume_punc(":")(input)?;
    let (input, typ) = parse_type(input)?;
    let (input, _info) = try_consume_info(input)?;
    Ok((input, CircuitComponent::Wire(name.to_string(), typ)))
}

fn parse_connectlike<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Statement, ParseErr> {
    map(alt((
        parse_connect,
        parse_invalidate,
    )), |connectlike| Statement::Connectlike(connectlike))(input)
}

fn parse_connect<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Connectlike, ParseErr> {
    let (input, _) = consume_keyword("connect")(input)?;
    let (input, r) = parse_reference(input)?;
    let (input, _) = consume_punc(",")(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((input, Connectlike::Connect(r, Box::new(expr))))
}

fn parse_invalidate<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Connectlike, ParseErr> {
    let (input, _) = consume_keyword("invalidate")(input)?;
    let (input, r) = parse_reference(input)?;
    Ok((input, Connectlike::Invalidate(r)))
}
