use firrtlizer::tokenizer::tokenize;

fn main() {
    let input = std::fs::read_to_string("Top.fir").unwrap();
    let toks = tokenize(&input).unwrap();
    for tok in toks {
        println!("{tok:?}");
    }
}
