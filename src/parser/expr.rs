use super::*;
use crate::Expr;
use nom::IResult;
use nom::branch::alt;
use crate::RefPath;

pub(crate) fn parse_expr<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Expr, ParseErr> {
    alt((
        parse_expr_var,
    ))(input)
}

fn parse_expr_var<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Expr, ParseErr> {
    let (input, refpath) = parse_reference(input)?;
    Ok((input, Expr::Var(refpath)))
}
