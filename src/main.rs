use firrtlizer::tokenizer::Tok;
use firrtlizer::tokenizer::tokenize;
use firrtlizer::parser::parse;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let default = "Top.fir".to_string();
    let filename = argv.get(1).unwrap_or(&default);
    let input = std::fs::read_to_string(filename).unwrap();

    let toks = tokenize(&input).unwrap();
    print_toks(&toks);
//    let circuit = parse(&toks).unwrap();
//    dbg!(circuit);
}

fn print_toks(toks: &[Tok]) {
    for tok in toks {
        print!("{tok:?} ");
        if *tok == Tok::Newline {
            println!();
        }
    }
    println!();
}
