use super::*;

/// Make sure FIRST(A) is defined in the "obvious" case.
#[test]
fn test_first() {
    let grammar = Grammar::new()
        .nonterminal("A")
        .terminal("x")
        .terminal("y")
        .rule("A", &["x"])
        .rule("A", &["y"])
        .build();

    let a = grammar.symbol("A").unwrap();
    let x = grammar.symbol("x").unwrap();
    let y = grammar.symbol("y").unwrap();

    assert_eq!(a.firsts(), [x, y].iter().copied().collect());
}

/// Make sure NULLABLE(A) is defined.
/// Ensure that it handles both the immediate case and the recursive case.
#[test]
fn test_empty() {
    let grammar = Grammar::new()
        .nonterminal("A")
        .nonterminal("B")
        .nonterminal("C")
        .terminal("x")
        .rule("A", &["x"])
        .rule("A", &[])
        .rule("B", &["A"])
        .rule("C", &["x"])
        .build();

    let a = grammar.symbol("A").unwrap();
    let b = grammar.symbol("B").unwrap();
    let x = grammar.symbol("x").unwrap();

    assert_eq!(grammar.nullables(), [a, b].iter().copied().collect());
}

/// Make sure FIRST(A) handles the case where the first symbol of the RHS of a rule is nullable.
#[test]
fn test_first_with_empty() {
    let grammar = Grammar::new()
        .nonterminal("A")
        .nonterminal("B")
        .terminal("x")
        .terminal("x")
        .rule("A", &["x"])
        .rule("A", &[])
        .rule("B", &["A", "x"])
        .build();

    let a = grammar.symbol("A").unwrap();
    let b = grammar.symbol("B").unwrap();
    let x = grammar.symbol("x").unwrap();

    assert_eq!(a.firsts(), [x].iter().copied().collect());
    assert_eq!(b.firsts(), [x].iter().copied().collect());
}

/// Test FIRST even when you have left recursion.
#[test]
fn test_first_left_recursion() {
    let grammar = Grammar::new()
        .nonterminal("A")
        .nonterminal("B")
        .terminal("x")
        .terminal("x")
        .rule("A", &["x"])
        .rule("A", &["A", "x"])
        .build();

    let a = grammar.symbol("A").unwrap();
    let x = grammar.symbol("x").unwrap();

    assert_eq!(a.firsts(), [x].iter().copied().collect());
}

/// Test FIRST even when you have mutual recursion.
#[test]
fn test_first_mutual_recursion() {
    let grammar = Grammar::new()
        .nonterminal("A")
        .nonterminal("B")
        .terminal("x")
        .terminal("x")
        .rule("A", &["x"])
        .rule("A", &["B"])
        .rule("B", &["A"])
        .build();

    let a = grammar.symbol("A").unwrap();
    let x = grammar.symbol("x").unwrap();

    assert_eq!(a.firsts(), [x].iter().copied().collect());
}

#[test]
fn test_follow_simple() {
    let grammar = Grammar::new()
        .nonterminal("A")
        .nonterminal("B")
        .terminal("x")
        .terminal("y")
        .rule("A", &["x"])
        .rule("B", &["A", "x"])
        .rule("B", &["A", "y"])
        .build();

    let a = grammar.symbol("A").unwrap();
    let x = grammar.symbol("x").unwrap();
    let y = grammar.symbol("y").unwrap();

    assert_eq!(a.follows(), [x, y].iter().copied().collect());
}

#[test]
fn test_follow_nullable() {
    let grammar = Grammar::new()
        .nonterminal("A")
        .nonterminal("B")
        .nonterminal("C")
        .terminal("x")
        .terminal("y")
        .terminal("z")
        .rule("A", &["x"])
        .rule("B", &["A", "y"])
        .rule("B", &["A", "C", "z"])
        .rule("C", &[])
        .build();

    let a = grammar.symbol("A").unwrap();
    let x = grammar.symbol("x").unwrap();
    let y = grammar.symbol("y").unwrap();
    let z = grammar.symbol("z").unwrap();

    assert_eq!(a.follows(), [y, z].iter().copied().collect());
}

#[test]
fn test_first_nullable() {
    let grammar = Grammar::new()
        .nonterminal("A")
        .terminal("x")
        .rule("A", &["A", "x"])
        .rule("A", &[])
        .build();

    let a = grammar.symbol("A").unwrap();
    let x = grammar.symbol("x").unwrap();

    assert_eq!(a.firsts(), [x].iter().copied().collect());
}

#[test]
fn foo() {
    let grammar = Grammar::new()
        .nonterminal("start")
        .nonterminal("circuit")
        .nonterminal("{decl}")
        .nonterminal("decl")

        .terminal("KW_CIRCUIT")
        .terminal("KW_MODULE")
        .terminal("NEWLINE")
        .terminal("DEDENT")
        .terminal("INDENT")
        .terminal("VERSION")
        .terminal("ID")
        .terminal("INFO")
        .terminal("COMMA")
        .terminal("COLON")
        .terminal("EQ")
        .terminal("DOT")
        .terminal("STRING")

        .rule("start", &["circuit"])
        .rule("circuit", &["VERSION", "NEWLINE", "KW_CIRCUIT", "ID", "INFO", "COLON", "NEWLINE", "INDENT", "DEDENT"])
        .rule("circuit", &["VERSION", "NEWLINE", "KW_CIRCUIT", "ID", "COLON", "NEWLINE", "INDENT", "{decl}", "DEDENT"])
        .rule("{decl}", &[])
        .rule("{decl}", &["{decl}", "decl"])
        .rule("decl", &["KW_MODULE"])

        .build();

    for symbol in grammar.symbols() {
        symbol.firsts();
    }

    let decl_star = grammar.symbol("{decl}").unwrap();
    let kw_module = grammar.symbol("KW_MODULE").unwrap();
    let dedent = grammar.symbol("DEDENT").unwrap();

    assert_eq!(grammar.nullables(), vec![decl_star].into_iter().collect());

    assert!(decl_star.is_nullable());
    assert_eq!(decl_star.firsts(), vec![kw_module, dedent].into_iter().collect());
}
