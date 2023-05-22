use firrtlizer::parser::TermParser;
use firrtlizer::lexer::Lexer;

fn main() {
    let parser = TermParser::new();
    let input = std::fs::read_to_string("Top.fir").unwrap();
    let val = parser.parse(Lexer::new(input));
    println!("{val:?}");
}
