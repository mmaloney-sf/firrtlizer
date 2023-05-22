use std::io::{Read, BufReader};
pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;
use regex::Regex;

#[derive(Clone, Debug)]
pub enum Tok {
    Unknown,
    Indent,
    Dedent,
    Newline,
    Ident(String),
    Lit(u64),
    Const,
    Circuit,
    Flip,
    Wire,
    Reg,
    Inst,
    Mod,
}

#[derive(Debug)]
pub enum LexicalError {
}

#[derive(PartialOrd, Ord, Debug, Eq, PartialEq, Clone, Default, Copy)]
pub struct Loc(usize, usize);

impl Loc {
    pub fn new(line: usize, col: usize) -> Loc {
        Loc(line, col)
    }

    pub fn line(&self) -> usize {
        self.0
    }

    pub fn col(&self) -> usize {
        self.1
    }
}

fn leading_spaces(line: &[u8]) -> usize {
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

pub struct Lexer {
    loc_toks: Vec<(Loc, Tok, Loc)>,
    index: usize,
}

fn consume_keyword<'a>(line: &mut [u8], keyword: &str) -> bool {
    todo!()
        /*
    let keyword_bytes = keyword.as_bytes();

    if line.starts_with(keyword_bytes) {
        if line.len() == keyword_bytes.len() {
            *line = line[keyword_bytes.len()..];
            return true;
        } else if line[keyword_bytes.len()] == ' ' as u8 {
            &line[keyword_bytes.len()..]
        } else {
            line
        }
    } else {
        line
    }
    */
}

fn consume_whitespace<'a>(line: &'a [u8]) -> &'a [u8] {
    let mut i = 0;
    while i < line.len() && line[i] == ' ' as u8 {
        i += 1;
    }
    &line[i..]
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let loc_toks: Vec<(Loc, Tok, Loc)> = vec![];

        let mut result = Lexer {
            loc_toks,
            index: 0,
        };

        for (lineno, line) in input.lines().enumerate() {
            let line_loc_toks = Lexer::tokenize_line(&line.as_bytes(), lineno, 0);
            result.loc_toks.extend_from_slice(&line_loc_toks);
        }

        result
    }

    fn tokenize_line(line: &[u8], lineno: usize, current_indent: usize) -> Vec<(Loc, Tok, Loc)> {
        todo!()
            /*
        let mut loc_toks = vec![];
        let leading_spaces = leading_spaces(line);
        let loc = Loc::new(lineno, 0);
        if leading_spaces > current_indent {
            loc_toks.push((loc.clone(), Tok::Indent, loc));
        } else if leading_spaces < current_indent {
            loc_toks.push((loc.clone(), Tok::Indent, loc));
        }

        let re_flip = Regex::new(r"flip( (.*)|$)").unwrap();
        let re_circuit = Regex::new(r"circuit( (.*)|$)").unwrap();
        let mut col = 0;

        while col < line.len() {
            let start_loc = Loc::new(lineno, col);
            if re_flip.is_match(&line[col..]) {
                loc_toks.push((start_loc, Tok::Flip, start_loc));
            } else if re_circuit.is_match(&line[col..]) {
                loc_toks.push((start_loc, Tok::Circuit, start_loc));
            } else {
                loc_toks.push((start_loc, Tok::Unknown, start_loc));
            }
            col += 1;
        }
        let loc = Loc::new(lineno, line.len());
        loc_toks.push((loc.clone(), Tok::Newline, loc));
        loc_toks
    */
    }
}

impl Iterator for Lexer {
    type Item = Spanned<Tok, Loc, LexicalError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.loc_toks.len() {
            let loc_tok = self.loc_toks[self.index].clone();
            self.index += 1;
            println!("{:?}", &loc_tok.1);
            Some(Ok(loc_tok))
        } else {
            None
        }
    }
}
