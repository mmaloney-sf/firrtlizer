use tokenizer::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let src = if args.len() < 2 {
    "
circuit Main :
    public module Top :
        input  inp : UInt<1>
        output out : UInt<1>
        connect out, inp
        ".to_string()
    } else {
        std::fs::read_to_string(&args[1]).unwrap()
    };

    let limit = match args.get(2) {
        Some(arg) => arg.parse::<usize>().unwrap(),
        None => usize::MAX,
    };

    let mut lex = FirrtlLexer::new(&src);

    let mut i = 0;
    let mut level = 0;
    while let Some(Ok(token)) = lex.next() {
        if i < limit {
            i += 1;
        } else {
            println!();
            break;
        }
        if token == Token::Newline {
            let indent = " ".repeat(level);
            print!("{token:?}\n{indent}");
        } else if token == Token::Indent {
            level += 4;
            let indent = " ".repeat(level);
            print!("{token:?}\n{indent}");
        } else if token == Token::Dedent {
            level -= 4;
            let indent = " ".repeat(level);
            print!("{token:?}\n{indent}");
        } else {
            print!("{token:?}({:?}) ", lex.slice());
        }
    }
}

