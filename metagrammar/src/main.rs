use metagrammar::GrammarParser;

lalrpop_util::lalrpop_mod!(metagrammar);

pub struct Grammar {
    rules: Vec<Rule>,
}

pub struct Rule {
    lhs: Symbol,
    rhs: SymbolExpr,
}

pub enum SymbolExpr {
    Alt(Vec<SymbolExpr>),
    Seq(Vec<SymbolExpr>),
    Term(Symbol),
    Nonterm(Symbol),
    Plus(Box<SymbolExpr>),
    Star(Box<SymbolExpr>),
    Opt(Box<SymbolExpr>),
}

pub type Symbol = String;

fn main() {
    let grammar = include_str!("../../GRAMMAR");
    let result = GrammarParser::new().parse(&grammar).unwrap();
}
