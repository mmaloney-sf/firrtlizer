use super::*;
use crate::Expr;
use nom::IResult;
use nom::branch::alt;
use crate::RefPath;
use crate::Value;

pub(crate) fn parse_expr<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Expr, ParseErr> {
    alt((
        parse_expr_var,
        parse_expr_lit,
        parse_expr_primop,
    ))(input)
}

fn parse_expr_var<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Expr, ParseErr> {
    let (input, refpath) = parse_reference(input)?;
    Ok((input, Expr::Var(refpath)))
}

fn parse_expr_primop<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Expr, ParseErr> {
    let (primop_name, _) = consume_id(input)?;
    eprintln!("Primop: {primop_name:?}");
    let (input, _) = consume_punc("(")(input)?;
    let (input, exprs) = separated_list0(consume_punc(","), parse_expr)(input)?;
    if let Some(Tok::Punc(",")) = input.get(0) {
        let (input, _) = consume_punc(",")(input)?;
        let (input, vs) = separated_list0(consume_punc(","), consume_lit)(input)?;
    }
    let (input, _) = consume_punc(")")(input)?;
    Ok((input, Expr::Var("asdf".into())))
}

fn parse_expr_lit<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Expr, ParseErr> {
    eprintln!("parse_expr_lit()");
    let (input, _) = consume_keyword("UInt")(input)?;
    let (input, width) = opt(parse_expr_width)(input)?;
    let (input, _) = consume_punc("(")(input)?;
    let (input, v) = alt((
         consume_lit,
         |i| { consume_id(i).map(|(r, t)| (r, 0)) },
    ))(input)?;
    let (input, _) = consume_punc(")")(input)?;
    Ok((input, Expr::Var("asdf".into())))
}

#[test]
fn test_parse_expr_lit() {
    let typ = "UInt<8>(0)";
    let toks: Vec<Tok> = crate::tokenizer::tokenize(typ).unwrap();
    let toks = &toks[..toks.len()-1];
    parse_expr(toks).unwrap();

    let typ = r#"UInt<1>(0h0)"#;
    let toks: Vec<Tok> = crate::tokenizer::tokenize(typ).unwrap();
    let toks = &toks[..toks.len()-1];
    parse_expr(toks).unwrap();
}

fn parse_expr_width<'a: 'b, 'b>(input: &'b [Tok<'a>]) -> IResult<&'b [Tok<'a>], Expr, ParseErr> {
    let (input, _) = consume_punc("<")(input)?;
    let (input, v) = consume_lit(input)?;
    let (input, _) = consume_punc(">")(input)?;
    Ok((input, Expr::Lit(Value::UInt(0, v))))
}
