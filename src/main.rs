use firrtlizer::tokenizer::Tok;
use firrtlizer::tokenizer::tokenize;
use firrtlizer::parser::{parse, parse_decl};

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let default = "Top.fir".to_string();
    let filename = argv.get(1).unwrap_or(&default);
    let input = std::fs::read_to_string(filename).unwrap();

    let toks = tokenize(&input).unwrap();
    let first_module_toks = first_module(&toks);
    print_toks(first_module_toks);

    let decl = parse_decl(&first_module_toks).unwrap();
    dbg!(decl);

//    let circuit = parse(&first_module_toks).unwrap();
//    dbg!(circuit);
}

fn print_toks(toks: &[Tok]) {
    for tok in toks {
        println!("{tok:?} ");
    }
    println!();
}

fn first_module<'a, 'b>(toks: &'a [Tok<'b>]) -> &'a [Tok<'b>] {
    let mut start = 0;
    while let Some(tok) = toks.get(start) {
        if *tok == Tok::Id("module") {
            break;
        }
        start += 1;
    }

    let mut end = start + 1;
    while let Some(tok) = toks.get(end) {
        if *tok == Tok::Id("module") {
            break;
        }
        end += 1;
    }
    &toks[start..end]
}
