use parsing::*;

use std::{any::Any, collections::{HashMap, HashSet}, ops::Deref, rc::Rc};

pub type State<'a> = ItemSet<'a>;

#[derive(Clone)]
struct Node<'a>(Rc<NodeData<'a>>);

#[derive(Debug)]
struct NodeData<'a> {
    symbol: Symbol<'a>,
    children: Vec<Node<'a>>,
}

impl<'a> std::ops::Deref for Node<'a> {
    type Target = NodeData<'a>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<'a> From<NodeData<'a>> for Node<'a> {
    fn from(value: NodeData<'a>) -> Self {
        Node(Rc::new(value))
    }
}

impl<'a> std::fmt::Debug for Node<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.symbol)
    }
}

pub type StateIndex = usize;

pub struct ParseTable<'a> {
    grammar: &'a Grammar,
    states: Vec<ItemSet<'a>>,
    actions: HashMap<(StateIndex, Option<Symbol<'a>>), Vec<Action<'a>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<'a> {
    Shift(StateIndex),
    Reduce(Rule<'a>),
    Halt,
}

impl<'a> ParseTable<'a> {
    #[tracing::instrument(skip_all)]
    pub fn new(grammar: &'a Grammar) -> ParseTable<'a> {
        tracing::info!("Here");
        let states = Self::build_states(&grammar);
        let actions = Self::build_actions(&grammar, &states);

        ParseTable {
            grammar,
            states,
            actions,
        }
    }

    fn build_states(grammar: &'a Grammar) -> Vec<State<'a>> {
        let mut states = vec![];

        let start_state = ItemSet::singleton(grammar.start_rule().item(0));

        let mut states_remaining = vec![start_state];

        while let Some(state) = states_remaining.pop() {
            for symbol in grammar.symbols() {
                let next_state = state.follow(symbol);
                if next_state.is_empty() {
                    continue;
                }

                if !states.contains(&next_state) {
                    states_remaining.push(next_state);
                }
            }

            states.push(state);
        }

        states
    }

    fn build_actions(grammar: &'a Grammar, states: &[State<'a>]) -> HashMap<(StateIndex, Option<Symbol<'a>>), Vec<Action<'a>>> {
        let mut actions = HashMap::new();
        for (i, _src_state) in states.iter().enumerate() {
            for symbol in grammar.symbols() {
                actions.insert((i, Some(symbol)), vec![]);
            }
            actions.insert((i, None), vec![]);
        }

        for (src_state_index, src_state) in states.iter().enumerate() {
            for src_item in src_state.items() {
//                let spn = tracing::span!(target="FOO", tracing::Level::INFO, "state", state=src_state_index);
//                tracing::event!(tracing::Level::INFO, state = src_state_index, "State = {src_state_index}");
                match src_item.next_symbol() {
                    Some(symbol) => {
                        let dst_state = src_state.follow(symbol);
                        let dst_state_index = Self::state_index(&dst_state, &states);
                        let mut actions = actions.get_mut(&(src_state_index, Some(symbol))).unwrap();
                        let action = Action::Shift(dst_state_index);
                        if !actions.contains(&action) {
                            actions.push(action);
                        }
                    }
                    None => {
                        for symbol in src_item.rule().lhs().follows() {
                            let mut actions = actions.get_mut(&(src_state_index, Some(symbol))).unwrap();
                            actions.push(Action::Reduce(src_item.rule()));
                        }

                        // End of input
                        let mut actions = actions.get_mut(&(src_state_index, None)).unwrap();
                        actions.push(Action::Reduce(src_item.rule()));
                        if src_state_index == 7 {
                            tracing::info!("actions: {actions:?}");
                        }
                    }
                }
            }
        }

        actions.get_mut(&(0, Some(grammar.start_rule().lhs()))).unwrap().insert(0, Action::Halt);

        actions
    }

    fn state_index(itemset: &ItemSet, itemsets: &[ItemSet]) -> usize {
        itemsets
            .iter()
            .enumerate()
            .find_map(|(j, st)| if itemset == st { Some(j) } else { None })
            .unwrap()
    }
}

pub struct Machine<'a, 'b> {
    parse_table: &'b ParseTable<'a>,
    head: Vec<Symbol<'a>>,
    stack: Vec<(StateIndex, Symbol<'a>)>,
    halted: bool,
    step: usize,
}

impl<'a, 'b> Machine<'a, 'b> {
    pub fn new(parse_table: &'b ParseTable<'a>) -> Machine<'a, 'b> {
        Machine {
            parse_table,
            head: vec![],
            stack: vec![],
            halted: false,
            step: 0,
        }
    }

    fn state(&self) -> StateIndex {
        self.stack.last().map(|(state_index, _sym)| *state_index).unwrap_or(0)
    }

    fn step(&mut self, symbol: Option<Symbol<'a>>) {
        let state = self.state();

        eprintln!("STEP:   {:?}", self.step);
        eprintln!("SYMBOL: {:?}", symbol);
        eprintln!("STACK:  {:?}", &self.stack);
        eprintln!("STATE:  {:?}", state);
        let state_rep = format!("{:?}", &self.parse_table.states[state]);
        for line in state_rep.lines() {
            eprintln!("    {line}");
        }

        let actions = &self.parse_table.actions.get(&(state, symbol));

        if let Some(actions) = actions {
            assert_eq!(actions.len(), 1);
            let action = actions[0];

            match action {
                Action::Shift(dst_state_index) => {
                    eprintln!("ACTION: SHIFT {}", dst_state_index);
                    self.stack.push((dst_state_index, symbol.unwrap()));
                }
                Action::Reduce(rule) => {
                    eprintln!("ACTION: REDUCE {:?}", rule);

                    self.head.insert(0, rule.lhs());

                    if let Some(symbol) = symbol {
                        self.head.insert(0, symbol);
                    }

                    let mut children = vec![];

                    for _ in 0..rule.rhs().len() {
                        let Some((_state, sym)) = self.stack.pop() else { panic!() };
                        children.insert(0, sym);
                    }
                }
                Action::Halt => {
                    eprintln!("ACTION: HALT");
                    self.halted = true;
                }
            }
        }
        eprintln!();
        self.step += 1;
    }

    fn run(&mut self, input: &mut impl Iterator<Item=Symbol<'a>>) {
        while !self.halted {
            if let Some(symbol) = self.head.pop() {
                self.step(Some(symbol));
            } else {
                let symbol = input.next();
                self.step(symbol);
            }
        }
    }
}

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
        .rule("decl", &["KW_MODULE", "ID", "COLON", "NEWLINE", "INDENT", "ID", "NEWLINE", "DEDENT"])

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
}
