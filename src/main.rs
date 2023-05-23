use firrtlizer::tokenizer::Tok;
use firrtlizer::tokenizer::tokenize;
use firrtlizer::parser::parse;

fn main() {
    let input = std::fs::read_to_string("Top.fir").unwrap();
    let toks = tokenize(&input).unwrap();
    for tok in &toks {
        print!("{tok:?} ");
        if *tok == Tok::Newline {
            println!();
        }
    }
    println!();
    let circuit = parse(&toks).unwrap();
    dbg!(circuit);
}
