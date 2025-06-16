use parsing::*;
use parsing::lr0::*;

use std::{any::Any, collections::{HashMap, HashSet}, ops::Deref, rc::Rc};

fn main() {
    let file = std::fs::File::create("log.txt").expect("Could not create log file");
//    let writer = BufWriter::new(file);

    let subscriber = tracing_subscriber::FmtSubscriber::builder()
//        .with_max_level(tracing::Level::TRACE)
//        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
//        .with_writer(std::io::stderr)
        .with_writer(file)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let grammar = Grammar::new()
        .symbol("start")
        .symbol("circuit")
        .symbol("{decl}")
        .symbol("decl")

        .symbol("KW_CIRCUIT")
        .symbol("KW_MODULE")
        .symbol("KW_SKIP")
        .symbol("KW_INPUT")
        .symbol("KW_OUTPUT")
        .symbol("KW_PUBLIC")
        .symbol("NEWLINE")
        .symbol("DEDENT")
        .symbol("INDENT")
        .symbol("VERSION")
        .symbol("ID")
        .symbol("INFO")
        .symbol("COMMA")
        .symbol("COLON")
        .symbol("EQ")
        .symbol("DOT")
        .symbol("STRING")

        .rule("start", &["circuit"])
        .rule("circuit", &["VERSION", "NEWLINE", "KW_CIRCUIT", "ID", "INFO", "COLON", "NEWLINE", "INDENT", "DEDENT"])
        .rule("circuit", &["VERSION", "NEWLINE", "KW_CIRCUIT", "ID", "COLON", "NEWLINE", "INDENT", "{decl}", "DEDENT"])
        .rule("{decl}", &[])
        .rule("{decl}", &["{decl}", "decl"])
        .rule("decl", &["decl_module"])
        .rule("decl_module", &["[public]", "KW_MODULE", "ID", "{enablelayer}", "COLON", "[info]", "NEWLINE", "INDENT", "SKIP", "{stmt}", "DEDENT"])
        .rule("[public]", &[])
        .rule("[public]", &["KW_PUBLIC"])

        .build();

    tracing::info!("Built grammar");

//    circuit = version , newline ,
//"circuit" , id , ":" , [ annotations ] , [ info ] , newline , indent ,
//{ decl } ,
//dedent ;

    eprintln!("Grammar:");
    eprintln!("{grammar:?}");
    eprintln!();

    tracing::info!("Nullables");
    eprintln!("Nullables: {:?}", grammar.nullables());

    let table = ParseTable::new(&grammar);

    eprintln!("Parse Table");
    for state in 0..table.states.len() {
        eprintln!("    State {state}");
        for symbol in table.grammar.symbols() {
            eprintln!("        on {symbol} => {:?}", &table.actions[&(state, Some(symbol))]);
        }
        eprintln!("        on $ => {:?}", &table.actions[&(state, None)]);
        eprintln!();
    }
    eprintln!();

    let source = std::fs::read_to_string(&std::env::args().skip(1).next().unwrap()).unwrap();

    let lex = tokenizer::FirrtlLexer::new(&source);

    /*
    eprintln!("Execute:");
    let mut input = lex
        .into_iter()
        .map(|token| {
            let token = token.unwrap();
            let s = match token {
                tokenizer::Token::Lex(lex_token) => {
                    match lex_token {
                        tokenizer::LexToken::Version => "VERSION",
                        tokenizer::LexToken::Newline(_) => "NEWLINE",
                        tokenizer::LexToken::KwCircuit => "KW_CIRCUIT",
                        tokenizer::LexToken::Id => "ID",
                        tokenizer::LexToken::Info => "INFO",
                        tokenizer::LexToken::CurlyLeft => "CURLYLEFT",
                        tokenizer::LexToken::CurlyRight => "CURLYRIGHT",
                        tokenizer::LexToken::BracketLeft => "BRACKETlEFT",
                        tokenizer::LexToken::BracketRight => "BRACKETRIGHT",
                        tokenizer::LexToken::ParenLeft => "PARENLEFT",
                        tokenizer::LexToken::ParenRight => "PARENRIGHT",
                        tokenizer::LexToken::AngLeft => "ANGLEFT",
                        tokenizer::LexToken::AngRight => "ANGRIGHT",
                        tokenizer::LexToken::Comma => "COMMA",
                        tokenizer::LexToken::Colon => "COLON",
                        tokenizer::LexToken::Eq => "EQ",
                        tokenizer::LexToken::Dot => "DOT",
                        tokenizer::LexToken::String => "STRING",
                        tokenizer::LexToken::KwModule => "KW_MODULE",
                        tokenizer::LexToken::KwSkip => "KW_SKIP",
                        tokenizer::LexToken::KwInput => "KW_INPUT",
                        tokenizer::LexToken::KwOutput => "KW_OUTPUT",
                        tokenizer::LexToken::Comment => unreachable!(),
                    }
                },
                tokenizer::Token::Newline => "NEWLINE",
                tokenizer::Token::Indent => "INDENT",
                tokenizer::Token::Dedent => "DEDENT",
            };
            grammar.symbol(s).expect(&format!("Could not find symbol {s:?}"))
        });
    let mut machine = Machine::new(&table);
    machine.run(&mut input);
*/
}
