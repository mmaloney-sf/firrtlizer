use std::{collections::HashMap, rc::Rc};

use crate::*;

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

#[derive(Debug)]
pub struct ParseTable<'a> {
    pub grammar: &'a Grammar,
    pub states: Vec<ItemSet<'a>>,
    pub actions: HashMap<(StateIndex, Option<Symbol<'a>>), Vec<Action<'a>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<'a> {
    Shift(StateIndex),
    Reduce(Rule<'a>),
    Halt,
}

impl<'a> ParseTable<'a> {
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

        // We crated a start state for the non-terminal that appears in the first rule.
        // TODO - This should be a synthetic state, IMO.
        let start_state = ItemSet::singleton(grammar.start_rule().item(0));

        let mut states_remaining = vec![start_state];

        // for each state
        while let Some(state) = states_remaining.pop() {

            // try following each symbol.
            // This pushes the . in the items, removing any items which go past the end of the rule.
            for symbol in grammar.symbols() {
                let next_state = state.follow(symbol);

                // ignore the "empty" state
                // this would be if we have an item with rules like `A -> B . a C`
                // but we received an input `b` which doesn't match and thus eliminates all rules.
                if next_state.is_empty() {
                    continue;
                }

                // don't allocate each state more than once.
                if !states.contains(&next_state) {
                    states_remaining.push(next_state);
                }
            }

            states.push(state);
        }

        states
    }

    fn build_actions(grammar: &'a Grammar, states: &[State<'a>]) -> HashMap<(StateIndex, Option<Symbol<'a>>), Vec<Action<'a>>> {
        // actions[(state_i, Some(symbol)] is a list of all Actions that can be taken 
        // when the machine is in state `state_i` and the nekxt input is `symbol`.
        // (`None` represents the end of input).
        let mut actions = HashMap::new();

        // Pre-allocate an empty list for all (state_i, maybe_symbol)-pairs.
        for (i, _src_state) in states.iter().enumerate() {
            for symbol in grammar.symbols() {
                actions.insert((i, Some(symbol)), vec![]);
            }
            actions.insert((i, None), vec![]);
        }

        for (src_state_index, src_state) in states.iter().enumerate() {
            for src_item in src_state.items() {
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

#[derive(Debug)]
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
            eprintln!("ACTIONS:  {:?}", &actions);
            // TODO
            // assert_eq!(actions.len(), 1, "Available actions: {actions:?}");
            let action: Action = if actions.len() == 0 {
                panic!("Machine halted unexpectedly");
            } else if actions.len() == 1 {
                 actions[0].clone()
            } else {
                *actions.into_iter().cloned().filter(|action| matches!(action, Action::Shift(_))).collect::<Vec<_>>().first().unwrap()
            };

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

    pub fn run(&mut self, input: &mut impl Iterator<Item=Symbol<'a>>) {
        let mut i = 0;
        while !self.halted {
            if let Some(symbol) = self.head.pop() {
                self.step(Some(symbol));
            } else {
                let symbol = input.next();
                eprintln!("READ SYMBOL: {symbol:?}  #{i}");
                i += 1;
                self.step(symbol);
            }
        }
    }
}
