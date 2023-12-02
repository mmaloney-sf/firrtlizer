use std::fs;
use std::path::Path;
use firrtlizer::parser::parse_statement;
use firrtlizer::tokenizer::tokenize;

// Assume you have a function named `foo()` defined somewhere
fn foo(input: &str) -> String {
    // Implement your logic for the `foo()` function
    // For example:
    input.to_lowercase()
}

#[test]
fn test_examples() {
    let contents = fs::read_to_string("tests/statements.txt").unwrap();
    for (i, line) in contents.lines().enumerate() {
        let lineno = i + 1;
        let toks = tokenize(line).unwrap();
        parse_statement(&toks).expect(&format!("Couldn't parse line {lineno}: {line}"));
    }
}
