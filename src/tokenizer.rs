use nom::IResult;
use nom::bytes::complete::{tag};
use nom::combinator::{value, eof};
use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::character::complete::{space0, space1, satisfy};
use nom::sequence::pair;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Tok<'a> {
    Indent(&'a str, usize),
    Dedent(&'a str, usize),
    Version(usize, usize, usize),
    Id(&'a str),
    Lit(u64),
    LitStr(&'a str),
    Info(&'a str),
    Punc(&'a str),
    Newline,
}

pub fn tokenize<'a>(input: &'a str) -> anyhow::Result<Vec<Tok<'a>>> {
    let mut indent_level: isize = 0;
    let spaces_per_indent_level: isize = 2;

    let mut toks = vec![];
    for (_i, line) in input.lines().enumerate() {
//        println!("processing like {i}");
        let num_leading_spaces = count_leading_spaces(line.as_bytes());
        let leading_spaces = &line[..num_leading_spaces];
        if num_leading_spaces > 0 {
            if num_leading_spaces > (indent_level * spaces_per_indent_level) as usize {
                let amount = num_leading_spaces - (indent_level * spaces_per_indent_level) as usize;
                toks.push(Tok::Indent(leading_spaces, amount));
                indent_level += 1;
            } else if num_leading_spaces < (indent_level * spaces_per_indent_level) as usize {
                let amount = (indent_level as isize * (spaces_per_indent_level - num_leading_spaces as isize)) as usize;
                toks.push(Tok::Dedent(leading_spaces, amount));
                indent_level -= 1;
            }
        }

        let line = &line[num_leading_spaces as usize..];
        let (_, line_toks) = tokenize_line(&line)?;
        for tok in line_toks {
            toks.push(tok);
        }
        toks.push(Tok::Newline);
    }
    for _ in 0..indent_level {
        let end_of_file: &str = &input[input.len()..];
        toks.push(Tok::Dedent(end_of_file, 0));
    }

    Ok(toks)
}

fn count_leading_spaces(line: &[u8]) -> usize {
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

fn tokenize_line<'a>(input: &'a str) -> IResult<&str, Vec<Tok<'a>>, ()> {
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

fn parse_token<'a>(input: &'a str) -> IResult<&str, Tok<'a>, ()> {
    let (input, tok) = alt((
        parse_token_version,
        parse_punc,
        parse_token_lp,
        parse_token_lit_num,
        parse_token_lit_str,
        parse_token_ident,
        parse_token_info,
    ))(input)?;
    let (input, _) = space0(input)?;
    Ok((input, tok))
}

fn parse_punc<'a>(input: &'a str) -> IResult<&str, Tok<'a>, ()> {
    let orig_input = input;
    let (input, s) = alt((
        tag("<="),
        tag("=="),
        tag(":"),
        tag("="),
        tag("."),
        tag(","),
    ))(input)?;
    let tok = Tok::Punc(&orig_input[..s.len()]);
    Ok((input, tok))
}

fn parse_token_lp<'a>(input: &'a str) -> IResult<&str, Tok<'a>, ()> {
    alt((
        value(Tok::Punc(&input[..1]), tag("<")),
        value(Tok::Punc(&input[..1]), tag(">")),
        value(Tok::Punc(&input[..1]), tag("(")),
        value(Tok::Punc(&input[..1]), tag(")")),
        value(Tok::Punc(&input[..1]), tag("[")),
        value(Tok::Punc(&input[..1]), tag("]")),
        value(Tok::Punc(&input[..1]), tag("{")),
        value(Tok::Punc(&input[..1]), tag("}")),
    ))(input)
}

fn parse_token_version<'a>(input: &'a str) -> IResult<&str, Tok<'a>, ()> {
    let (input, _) = tag("FIRRTL")(input)?;
    let (input, _) = space1(input)?;
    let (input, _) = tag("version")(input)?;
    let (input, _) = space1(input)?;
    let (input, major) = many1(satisfy(|ch| ch.is_numeric()))(input)?;
    let (input, _) = tag(".")(input)?;
    let (input, minor) = many1(satisfy(|ch| ch.is_numeric()))(input)?;
    let (input, _) = tag(".")(input)?;
    let (input, patch) = many1(satisfy(|ch| ch.is_numeric()))(input)?;
    let major = major.into_iter().collect::<String>();
    let minor = minor.into_iter().collect::<String>();
    let patch = patch.into_iter().collect::<String>();
    let tok = Tok::Version(major.parse().unwrap(), minor.parse().unwrap(), patch.parse().unwrap());
    Ok((input, tok))
}
//version = "FIRRTL" , "version" , sem_ver ;


fn parse_token_info<'a>(input: &'a str) -> IResult<&str, Tok<'a>, ()> {
    let orig_input = input;
    let (input, _) = tag("@[")(input)?;
    let (input, contents) = many0(satisfy(|ch| ch != ']'))(input)?;
    let (input, _) = tag("]")(input)?;
    let len = contents.len() + 3;
    Ok((input, Tok::Info(&orig_input[..len])))
}

fn parse_token_ident<'a>(input: &'a str) -> IResult<&str, Tok<'a>, ()> {
    let orig_input = &input;
    let (input, _head_char) = satisfy(|ch| ch.is_alphabetic() || ch == '_')(input)?;
    let (input, tail_chars) = many0(satisfy(|ch| ch.is_alphanumeric() || ch == '_'))(input)?;
    let len = 1 + tail_chars.len();
    let token = Tok::Id(&orig_input[..len]);
    Ok((input, token))
}

fn parse_token_lit_num<'a>(input: &'a str) -> IResult<&str, Tok<'a>, ()> {
    let orig_input = &input;
    let (input, number) = many1(satisfy(|ch| ch.is_numeric()))(input)?;
    let len = number.len();
    let number_str = &orig_input[..len];
    let token = Tok::Lit(number_str.parse().unwrap());
    Ok((input, token))
}

fn parse_token_lit_str<'a>(input: &'a str) -> IResult<&str, Tok<'a>, ()> {
    let orig_input = &input;
    let (input, _) = tag("\"")(input)?;
    let (input, contents) = many0(parse_token_lit_content_char)(input)?;
    let (input, _) = tag("\"")(input)?;
    let len = contents.len() + 2;
    let token = Tok::LitStr(&orig_input[..len]);
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
