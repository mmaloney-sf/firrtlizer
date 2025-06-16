use parsing::lr0;
use tokenizer::FirrtlLexer;
use std::collections::HashSet;

use metagrammar::GrammarParser;

lalrpop_util::lalrpop_mod!(metagrammar);

#[derive(Debug)]
pub struct Grammar {
    rules: Vec<Rule>,
}

impl Grammar {
    fn split(&mut self) {
        let mut rules_left = vec![];
        std::mem::swap(&mut self.rules, &mut rules_left);

        let mut rules = HashSet::new();

        while let Some(rule) = rules_left.pop() {
            if rule.is_simple() {
                rules.insert(rule);
            } else {
                for rule in rule.split() {
                    rules_left.push(rule);
                }
            }
        }

        let mut rules: Vec<_> = rules.into_iter().collect();
        std::mem::swap(&mut self.rules, &mut rules);
    }

    fn nonterminals(&self) -> HashSet<Symbol> {
        let mut result = HashSet::new();
        for rule in &self.rules {
            result.insert(rule.lhs.clone());
        }
        result
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    lhs: Symbol,
    rhs: SymbolExpr,
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {:?}", &self.lhs, &self.rhs)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum SymbolExpr {
    Alt(Vec<SymbolExpr>),
    Seq(Vec<SymbolExpr>),
    Term(Symbol),
    Nonterm(Symbol),
    Star(Box<SymbolExpr>),
    Opt(Box<SymbolExpr>),
    Group(Box<SymbolExpr>),
}

impl std::fmt::Debug for SymbolExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            SymbolExpr::Alt(es) => es.into_iter().cloned().map(|e| format!("{e:?}")).collect::<Vec<_>>().join(" | "),
            SymbolExpr::Seq(es) => {
                if es.is_empty() {
                    format!("SEQ()")
                } else {
                    format!("SEQ({})", es.into_iter().cloned().map(|e| format!("{e:?}")).collect::<Vec<_>>().join(" , "))
                }
            }
            SymbolExpr::Term(s) => s.clone(),
            SymbolExpr::Nonterm(s) => s.clone(),
            SymbolExpr::Star(e) => format!("{{ {e:?} }}"),
            SymbolExpr::Opt(e) => format!("[ {e:?} ]"),
            SymbolExpr::Group(e) => format!("({e:?})"),
        })
    }
}

impl SymbolExpr {
    // seqs of alts
    fn alt_of_seqs(&self) -> Vec<SymbolExpr> {
        match self {
            SymbolExpr::Alt(es) => {
                es.into_iter().cloned().map(|e| {
                    e
                }).collect()
            }
            SymbolExpr::Seq(es) => vec![self.clone()],
            _ => vec![SymbolExpr::Seq(vec![self.clone()])],
        }
    }

    fn needs_split(&self) -> bool {
        match self {
            SymbolExpr::Term(_) => false,
            SymbolExpr::Nonterm(_) => false,
            SymbolExpr::Alt(_) => true,
            SymbolExpr::Star(_) => true,
            SymbolExpr::Opt(_) => true,
            SymbolExpr::Group(e) => e.needs_split(),
            SymbolExpr::Seq(es) => es.iter().any(|e| e.needs_split()),
        }
    }

    fn is_compound(&self) -> bool {
        match self {
            SymbolExpr::Term(_) => false,
            SymbolExpr::Nonterm(_) => false,
            SymbolExpr::Alt(_) => true,
            SymbolExpr::Star(_) => true,
            SymbolExpr::Opt(_) => true,
            SymbolExpr::Group(e) => true,
            SymbolExpr::Seq(es) => es.iter().any(|e| e.needs_split()),
        }
    }

    fn definition(&self) -> Vec<Rule> {
        let lhs = format!("<{self:?}>");
        match self {
            SymbolExpr::Alt(es) => todo!(),
            SymbolExpr::Seq(_) => todo!("definition for {self:?}"),
            SymbolExpr::Term(_) => todo!(),
            SymbolExpr::Nonterm(_) => todo!(),
            SymbolExpr::Star(e) => vec![
                Rule {
                    lhs: lhs.clone(),
                    rhs: Self::Seq(vec![]),
                },
                Rule {
                    lhs: lhs.clone(),
                    rhs: Self::Seq(vec![SymbolExpr::Nonterm(lhs), *e.clone()]),
                }
            ],
            SymbolExpr::Opt(e) => vec![
                Rule {
                    lhs: lhs.clone(),
                    rhs: Self::Seq(vec![]),
                },
                Rule {
                    lhs,
                    rhs: *e.clone(),
                }
            ],
            SymbolExpr::Group(e) => vec![
                Rule {
                    lhs,
                    rhs: *e.clone(),
                }
            ],
        }
    }

    fn to_vec(&self) -> Vec<&'static str> {
        let mut result = vec![];
        if let SymbolExpr::Term(e) = self {
            result.push(e.clone().leak() as &'static str);
        } else if let SymbolExpr::Nonterm(e) = self {
            result.push(e.clone().leak() as &'static str);
        } else if let SymbolExpr::Seq(es) = self {
            for e in es {
                result.extend(e.to_vec());
            }
        } else {
            panic!("Can't to_vec: {self:?}");
        }
        result
    }
}

impl Rule {
    fn is_simple(&self) -> bool {
        !self.rhs.is_compound()
    }

    fn split(&self) -> Vec<Rule> {
        let mut result = vec![];

        for seq in self.rhs.alt_of_seqs() {
            let SymbolExpr::Seq(es) = &seq else { unreachable!() };
            let es: Vec<SymbolExpr> = es.iter().cloned().map(|e| {
                if e.is_compound() {
                    let new_e = format!("<{e:?}>");
                    result.extend(e.definition());
                    SymbolExpr::Nonterm(new_e)
                } else {
                    e
                }
            }).collect();
            let rule = Rule {
                lhs: self.lhs.clone(),
                rhs: SymbolExpr::Seq(es),
            };
            result.push(rule);
        }

        result
    }
}

pub type Symbol = String;

fn pos_to_line(s: &str, pos: usize) -> usize {
    let mut line = 1;
    for ch in s.chars().take(pos) {
        if ch == '\n' {
            line += 1;
        }
    }
    line
}

fn main() {
    let grammar_str = include_str!("../../GRAMMAR");
    let mut grammar_data = match GrammarParser::new().parse(&grammar_str) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{e:?}");
            match e {
                lalrpop_util::ParseError::InvalidToken { location } => todo!(),
                lalrpop_util::ParseError::UnrecognizedEof { location, expected } => todo!(),
                lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
                    let (start, token_text, end) = token;
                    let line_start = pos_to_line(grammar_str, start);
                    let line_end = pos_to_line(grammar_str, end);
                    eprintln!("Line: {line_start} (until {line_end}) token = {token_text:?}");
                }
                lalrpop_util::ParseError::ExtraToken { token } => todo!(),
                lalrpop_util::ParseError::User { error } => todo!(),
            }
            panic!();
        }
    };
//    dbg!(&grammar);
//    dbg!(&grammar.rules[0]);
//    dbg!(grammar.rules[0].rhs.alt_of_seqs());
//    dbg!(grammar.rules[0].split());
    grammar_data.split();
    dbg!(&grammar_data);

    let mut grammar = parsing::Grammar::new();

    grammar = grammar.symbol("START");

    for nonterminal in grammar_data.nonterminals() {
        eprintln!("{nonterminal:?}");
        grammar = grammar.symbol(nonterminal);
    }
    eprintln!();

    grammar = grammar.symbol("id");
    grammar = grammar.symbol("newline");
    grammar = grammar.symbol("indent");
    grammar = grammar.symbol("dedent");
    grammar = grammar.symbol(r#"".""#);
    grammar = grammar.symbol(r#"",""#);
    grammar = grammar.symbol(r#""(""#);
    grammar = grammar.symbol(r#"")""#);
    grammar = grammar.symbol(r#""<""#);
    grammar = grammar.symbol(r#"">""#);
    grammar = grammar.symbol(r#""{|""#);
    grammar = grammar.symbol(r#""|}""#);
    grammar = grammar.symbol(r#""[""#);
    grammar = grammar.symbol(r#""]""#);
    grammar = grammar.symbol(r#""{""#);
    grammar = grammar.symbol(r#""}""#);
    grammar = grammar.symbol(r#"":""#);
    grammar = grammar.symbol(r#""=""#);
    grammar = grammar.symbol(r#""=>""#);
    grammar = grammar.symbol(r#""<=""#);
    grammar = grammar.symbol(r#""data-type""#);
    grammar = grammar.symbol(r#""read-latency""#);
    grammar = grammar.symbol(r#""write-latency""#);
    grammar = grammar.symbol(r#""read-under-write""#);
    grammar = grammar.symbol(r#""readwriter""#);
    grammar = grammar.symbol(r#""writer""#);
    grammar = grammar.symbol(r#""reader""#);
    grammar = grammar.symbol(r#""formal""#);
    grammar = grammar.symbol(r#""layer""#);
    grammar = grammar.symbol(r#""attach""#);
    grammar = grammar.symbol(r#""depth""#);
    grammar = grammar.symbol(r#""invalidate""#);
    grammar = grammar.symbol(r#""invalid""#);
    grammar = grammar.symbol(r#""connect""#);
    grammar = grammar.symbol(r#""undefined""#);
    grammar = grammar.symbol(r#""new""#);
    grammar = grammar.symbol(r#""old""#);
    grammar = grammar.symbol(r#""mem""#);
    grammar = grammar.symbol(r#""enablelayer""#);
    grammar = grammar.symbol(r#""Probe""#);
    grammar = grammar.symbol(r#""RWProbe""#);
    grammar = grammar.symbol(r#""flip""#);
    grammar = grammar.symbol(r#""UInt""#);
    grammar = grammar.symbol(r#""SInt""#);
    grammar = grammar.symbol(r#""Analog""#);
    grammar = grammar.symbol(r#""Clock""#);
    grammar = grammar.symbol(r#""Reset""#);
    grammar = grammar.symbol(r#""AsyncReset""#);
    grammar = grammar.symbol(r#""Integer""#);
    grammar = grammar.symbol(r#""String""#);
    grammar = grammar.symbol(r#""List""#);
    grammar = grammar.symbol(r#""probe""#);
    grammar = grammar.symbol(r#""rwprobe""#);
    grammar = grammar.symbol(r#""read""#);
    grammar = grammar.symbol(r#""force""#);
    grammar = grammar.symbol(r#""force_initial""#);
    grammar = grammar.symbol(r#""release""#);
    grammar = grammar.symbol(r#""release_initial""#);
    grammar = grammar.symbol(r#""mux""#);
    grammar = grammar.symbol(r#""stop""#);
    grammar = grammar.symbol(r#""assert""#);
    grammar = grammar.symbol(r#""printf""#);
    grammar = grammar.symbol(r#""fprintf""#);
    grammar = grammar.symbol(r#""fflush""#);
    grammar = grammar.symbol(r#""const""#);
    grammar = grammar.symbol(r#""intrinsic""#);
    grammar = grammar.symbol(r#""skip""#);
    grammar = grammar.symbol(r#""layerblock""#);
    grammar = grammar.symbol(r#""module""#);
    grammar = grammar.symbol(r#""parameter""#);
    grammar = grammar.symbol(r#""defname""#);
    grammar = grammar.symbol(r#""extmodule""#);
    grammar = grammar.symbol(r#""of""#);
    grammar = grammar.symbol(r#""public""#);
    grammar = grammar.symbol(r#""type""#);
    grammar = grammar.symbol(r#""inst""#);
    grammar = grammar.symbol(r#""wire""#);
    grammar = grammar.symbol(r#""reg""#);
    grammar = grammar.symbol(r#""regreset""#);
    grammar = grammar.symbol(r#""node""#);
    grammar = grammar.symbol(r#""is""#);
    grammar = grammar.symbol(r#""input""#);
    grammar = grammar.symbol(r#""output""#);
    grammar = grammar.symbol(r#""cover""#);
    grammar = grammar.symbol(r#""assume""#);
    grammar = grammar.symbol(r#""match""#);
    grammar = grammar.symbol(r#""else""#);
    grammar = grammar.symbol(r#""when""#);
    grammar = grammar.symbol(r#""propassign""#);
    grammar = grammar.symbol(r#""define""#);
    grammar = grammar.symbol(r#""circuit""#);
    grammar = grammar.symbol("property_primop_varexpr_keyword");
    grammar = grammar.symbol("property_primop_2expr_keyword");
    grammar = grammar.symbol("primop_1expr2int_keyword");
    grammar = grammar.symbol("primop_1expr1int_keyword");
    grammar = grammar.symbol("primop_1expr_keyword");
    grammar = grammar.symbol("primop_2expr_keyword");
    grammar = grammar.symbol("type_constable");
    grammar = grammar.symbol("int");
    grammar = grammar.symbol("info");
    grammar = grammar.symbol("string_dq");
    grammar = grammar.symbol("string_sq");
    grammar = grammar.symbol("string");
    grammar = grammar.symbol("version");
    grammar = grammar.symbol("annotations");

    grammar = grammar.rule("START", &["circuit"]);

    for rule in &grammar_data.rules {
//        dbg!(&rule);
//        assert!(rule.is_simple());
        grammar = grammar.rule(&rule.lhs.clone(), &rule.rhs.to_vec());
    }

    let grammar = grammar.build();

    eprintln!("GRAMMAR:");
    eprintln!("{grammar:?}");
    eprintln!();

    dbg!(grammar.symbol("decl").unwrap().firsts());

    let table = lr0::ParseTable::new(&grammar);
    let mut machine = lr0::Machine::new(&table);

    let source = std::fs::read_to_string(&std::env::args().skip(1).next().unwrap()).unwrap();

    let lex = tokenizer::FirrtlLexer::new(&source);
    let mut input = massage_tokens(&grammar, lex.into_iter());
    machine.run(&mut input);

    eprintln!("DONE");
}

fn massage_tokens<'a>(grammar: &'a parsing::Grammar, lex: FirrtlLexer) -> impl Iterator<Item=parsing::Symbol<'a>> {
    lex
        .into_iter()
        .map(|token| {
            let token = token.unwrap();
            let s = match token {
                tokenizer::Token::Lex(lex_token) => {
                    match lex_token {
                        tokenizer::LexToken::Version => "version",
                        tokenizer::LexToken::KwCircuit => r#""circuit""#,
                        tokenizer::LexToken::Id => "id",
                        tokenizer::LexToken::Int => "int",
                        tokenizer::LexToken::Info => "info",
                        tokenizer::LexToken::CurlyLeft => r#""{""#,
                        tokenizer::LexToken::CurlyRight => r#""}""#,
                        tokenizer::LexToken::BracketLeft => r#""[""#,
                        tokenizer::LexToken::BracketRight => r#""]""#,
                        tokenizer::LexToken::ParenLeft => r#""(""#,
                        tokenizer::LexToken::ParenRight => r#"")""#,
                        tokenizer::LexToken::AngLeft => r#""<""#,
                        tokenizer::LexToken::AngRight => r#"">""#,
                        tokenizer::LexToken::Comma => r#"",""#,
                        tokenizer::LexToken::Colon => r#"":""#,
                        tokenizer::LexToken::Eq => r#""=""#,
                        tokenizer::LexToken::RevFatArrow => r#""<=""#,
                        tokenizer::LexToken::Dot => r#"".""#,
                        tokenizer::LexToken::String => "string",
                        tokenizer::LexToken::KwModule => r#""module""#,
                        tokenizer::LexToken::KwSkip => r#""skip""#,
                        tokenizer::LexToken::KwInput => r#""input""#,
                        tokenizer::LexToken::KwOutput => r#""output""#,
                        tokenizer::LexToken::Comment => unreachable!(),
                        tokenizer::LexToken::Newline(_) => unreachable!(),
                        tokenizer::LexToken::KwUInt => r#""UInt""#,
                        tokenizer::LexToken::KwSInt => r#""SInt""#,
                        tokenizer::LexToken::KwClock => r#""Clock""#,
                        tokenizer::LexToken::KwReset => r#""Reset""#,
                        tokenizer::LexToken::KwFlip => r#""flip""#,
                        tokenizer::LexToken::KwWire => r#""wire""#,
                        tokenizer::LexToken::KwReg => r#""reg""#,
                        tokenizer::LexToken::KwNode => r#""node""#,
                        tokenizer::LexToken::KwIs => r#""is""#,
                        tokenizer::LexToken::KwInvalid => r#""invalid""#,
                        tokenizer::LexToken::KwInvalidate => r#""invalidate""#,
                    }
                },
                tokenizer::Token::Newline => "newline",
                tokenizer::Token::Indent => "indent",
                tokenizer::Token::Dedent => "dedent",
            };
            grammar.symbol(s).expect(&format!("Could not find symbol {s:?}"))
        })
}
