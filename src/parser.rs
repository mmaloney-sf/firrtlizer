use crate::ast::*;

use nom::{Err, IResult};
use nom::bytes::complete::{tag};
use nom::combinator::{value, opt, eof};
use nom::branch::alt;
use nom::multi::{many0, many1};
use nom::character::complete::{space0, satisfy};

/*
pub struct Parser {
    indent_level: usize,
    spaces_per_ident_level: usize,
}

impl Parser {
    pub fn parse(input: &str) -> Circuit {
        let parser = Parser {
            indent_level: 0,
            spaces_per_ident_level: 2,
        };

        let lines: Vec<String> = input.to_string().lines().collect::<Vec<_>>();

        parser.parse_circuit(input)
    }

    fn parse_circuit(
}
*/
