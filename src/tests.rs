use crate::parser::parse_statement;
use crate::tokenizer::tokenize;

#[test]
fn test_examples() {
    let contents = std::fs::read_to_string("tests/statements.txt").unwrap();
    for (i, line) in contents.lines().enumerate() {
        let lineno = i + 1;
        let toks = tokenize(line).unwrap();
        eprintln!("{i}: {line}");
        parse_statement(&toks).expect(&format!("Couldn't parse line {lineno}: {line}"));
    }
}
