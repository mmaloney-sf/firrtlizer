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

        let mut rules = vec![];

        while let Some(rule) = rules_left.pop() {
            if rule.is_simple() {
                rules.push(rule);
            } else {
                for rule in rule.split() {
                    rules.push(rule);
                }
            }
        }

        std::mem::swap(&mut self.rules, &mut rules);
    }
}

#[derive(Clone)]
pub struct Rule {
    lhs: Symbol,
    rhs: SymbolExpr,
}

impl std::fmt::Debug for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} -> {:?}", &self.lhs, &self.rhs)
    }
}

#[derive(Clone)]
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
                    format!("(empty)")
                } else {
                    es.into_iter().cloned().map(|e| format!("{e:?}")).collect::<Vec<_>>().join(" , ")
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
            SymbolExpr::Seq(_) => todo!(),
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

fn main() {
    let grammar_str = include_str!("../../GRAMMAR");
    let mut grammar = GrammarParser::new().parse(&grammar_str).unwrap();
//    dbg!(&grammar);
//    dbg!(&grammar.rules[0]);
//    dbg!(grammar.rules[0].rhs.alt_of_seqs());
//    dbg!(grammar.rules[0].split());
    grammar.split();
    dbg!(&grammar);
}
