use std::collections::{HashMap, HashSet};

pub struct GrammarBuilder {
    pub(crate) symbols: Vec<SymbolData>,
    pub(crate) rules: Vec<RuleData>,
}

impl GrammarBuilder {
    pub fn symbol<S: Into<String>>(mut self, name: S) -> Self {
        self.symbols.push(SymbolData {
            name: name.into(),
        });
        self
    }

    pub fn rule(mut self, lhs: &str, rhs: &[&str]) -> Self {
        self.rules.push(RuleData {
            lhs: self.symbol_index(lhs),
            rhs: rhs.iter().map(|name| self.symbol_index(name)).collect(),
        });
        self
    }

    pub fn build(self) -> Grammar {
        Grammar {
            symbols: self.symbols,
            rules: self.rules,
        }
    }

    fn symbol_index(&self, name: &str) -> SymbolIndex {
        for (i, symbol_data) in self.symbols.iter().enumerate() {
            if symbol_data.name == name {
                return i;
            }
        }
        panic!("No such symbol: {name}")
    }
}

pub struct Grammar {
    pub(crate) symbols: Vec<SymbolData>,
    pub(crate) rules: Vec<RuleData>,
}

type SymbolIndex = usize;
type RuleIndex = usize;

#[derive(Clone, PartialEq, Eq)]
pub struct RuleData {
    pub(crate) lhs: SymbolIndex,
    pub(crate) rhs: Vec<SymbolIndex>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SymbolData {
    pub(crate) name: String,
}

#[derive(Clone, Copy)]
pub struct Rule<'a>(&'a Grammar, RuleIndex);

impl<'a> std::fmt::Debug for Rule<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ->", self.lhs())?;
        for sym in self.rhs() {
            write!(f, " {sym}")?;
        }
        Ok(())
    }
}

impl<'a> std::hash::Hash for Symbol<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index.hash(state)
    }
}

impl<'a> Rule<'a> {
    pub fn grammar(&self) -> &'a Grammar {
        self.0
    }

    pub fn index(&self) -> usize {
        self.1
    }

    pub fn name(&self) -> String {
        format!("{self:?}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Terminal,
    Nonterminal,
}

#[derive(Clone, Copy)]
pub struct Symbol<'a> {
    grammar: &'a Grammar,
    index: SymbolIndex,
}


impl<'a> PartialEq for Symbol<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.grammar, other.grammar) && self.index == other.index
    }
}

impl<'a> Eq for Symbol<'a> {}

impl<'a> Symbol<'a> {
    pub fn is_terminal(&self) -> bool {
        !self.grammar.rules().iter().any(|rule| rule.lhs() == *self)
    }

    pub fn is_nonterminal(&self) -> bool {
        self.grammar.rules().iter().any(|rule| rule.lhs() == *self)
    }

    pub fn as_str(&self) -> &str {
        self.grammar.symbols[self.index].name.as_str()
    }

    pub fn is_nullable(&self) -> bool {
        self.grammar.nullables().contains(self)
    }

    pub fn firsts(&self) -> HashSet<Symbol<'a>> {
        let ff = self.grammar.first_follows();
        ff.terminals_from(FFNode::First(*self)).into_iter().collect()
    }

    pub fn follows(&self) -> HashSet<Symbol<'a>> {
        let ff = self.grammar.first_follows();
        ff.terminals_from(FFNode::Follow(*self)).into_iter().collect()
    }
}

impl<'a> std::fmt::Display for Symbol<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = &self.grammar.symbols[self.index].name;
        write!(f, "{name}")
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Item<'a> {
    rule: Rule<'a>,
    pos: usize,
}

#[derive(Clone)]
pub struct ItemSet<'a>(&'a Grammar, Vec<Item<'a>>);

impl<'a> PartialEq for ItemSet<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0) && self.1 == other.1
    }
}

impl<'a> Eq for ItemSet<'a> {}

impl<'a> std::ops::Deref for Rule<'a> {
    type Target = RuleData;

    fn deref(&self) -> &Self::Target {
        let grammar = self.grammar();
        &grammar.rules[self.index()]
    }
}

impl<'a> PartialEq for Rule<'a> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0) && self.1 == other.1
    }
}

impl<'a> Eq for Rule<'a> {}

impl<'a> Rule<'a> {
    pub fn item(self, pos: usize) -> Item<'a> {
        assert!(pos <= self.rhs().len());
        Item {
            rule: self,
            pos,
        }
    }

    fn rule_data(&self) -> &RuleData {
        let grammar = self.grammar();
        &grammar.rules[self.1]
    }

    pub fn lhs(&self) -> Symbol<'a> {
        let grammar = self.grammar();
        let rule_data = self.rule_data();
        let lhs = rule_data.lhs;
        Symbol {
            grammar,
            index: lhs,
        }
    }

    pub fn rhs(&self) -> Vec<Symbol<'a>> {
        let grammar = self.grammar();
        let rule_data = self.rule_data();
        rule_data
            .rhs
            .iter()
            .copied()
            .map(|index|
                Symbol {
                    grammar,
                    index,
                }
            )
            .collect::<Vec<_>>()
    }
}

impl<'a> std::fmt::Debug for Symbol<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = &self.grammar.symbols[self.index].name;
        write!(f, "{name}")
    }
}

impl std::fmt::Debug for Grammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, rule) in self.rules().into_iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{rule:?}")?;
        }
        Ok(())
    }
}

impl<'a> std::fmt::Debug for Item<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let lhs = self.rule.lhs();
        let rhs = self.rule.rhs();
        write!(f, "{lhs} ->");
        for i in 0..self.pos {
            write!(f, " {:?}", &rhs[i])?;
        }
        write!(f, " .")?;
        for i in self.pos..rhs.len() {
            write!(f, " {:?}", &rhs[i])?;
        }
        Ok(())
    }
}

impl<'a> std::fmt::Debug for ItemSet<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, rule) in self.items().iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{rule:?}")?;
        }
        Ok(())
    }
}

impl<'a> Item<'a> {
    pub fn rule(&self) -> Rule<'a> {
        self.rule
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn grammar(&self) -> &'a Grammar {
        self.rule.grammar()
    }

    pub fn lhs(&self) -> Symbol<'a> {
        self.rule().lhs()
    }

    pub fn rhs(&self) -> Vec<Symbol<'a>> {
        self.rule().rhs()
    }

    pub fn next_symbol(&self) -> Option<Symbol<'a>> {
        if self.pos() < self.rhs().len() {
            Some(self.rhs()[self.pos()])
        } else {
            None
        }
    }

    pub fn step(&self) -> Item<'a> {
        Item {
            rule: self.rule, 
            pos: self.pos() + 1,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.pos() == self.rule().rhs().len()
    }
}

impl<'a> ItemSet<'a> {
    pub fn empty(grammar: &'a Grammar) -> ItemSet<'a> {
        ItemSet(grammar, vec![])
    }

    pub fn singleton(item: Item<'a>) -> ItemSet<'a> {
        let itemset = ItemSet(item.grammar(), vec![item]);
        itemset.closure()
    }

    pub fn is_empty(&self) -> bool {
        self.items().is_empty()
    }

    pub(crate) fn insert(&mut self, item: Item<'a>) -> bool {
        for search_item in self.items() {
            if search_item == &item {
                return false;
            }
        }
        self.1.push(item);
        true
    }

    pub fn follow(&self, sym: Symbol<'a>) -> ItemSet<'a> {
        let mut items = vec![];
        for item in self.items() {
            if let Some(search_sym) = item.next_symbol() {
                if search_sym == sym {
                    items.push(item.clone().step());
                }
            }
        }
        let grammar = self.grammar();
        let itemset = ItemSet(grammar, items);
        itemset.closure()
    }

    pub fn items(&self) -> &[Item<'a>] {
        self.1.as_slice()
    }

    pub(crate) fn closure(&self) -> ItemSet<'a> {
        let mut items_added = true;
        let mut nonterms_added: HashSet<Symbol<'a>> = HashSet::new();
        let mut itemset = self.items().to_vec();

        while items_added {
            let mut new_items = vec![];

            items_added = false;

            for item in &itemset {
                if let Some(symbol) = item.next_symbol() {
                    if symbol.is_nonterminal() {
                        if !nonterms_added.contains(&symbol) {
                            nonterms_added.insert(symbol);

                            for rule in self.grammar().rules_for(symbol) {
                                let item = rule.item(0);
                                new_items.push(item);
                                items_added = true;
                            }
                        }
                    }
                }
            }


            for item in new_items {
                if !itemset.contains(&item) {
                    itemset.push(item);
                }
            }
        }
        ItemSet(self.grammar(), itemset)
    }

    pub fn grammar(&self) -> &'a Grammar {
        self.0
    }
}

impl Grammar {
    pub fn new() -> GrammarBuilder {
        GrammarBuilder {
            symbols: vec![],
            rules: vec![],
        }
    }

    pub fn start_rule(&self) -> Rule {
        Rule(self, 0)
    }

    pub fn symbols(&self) -> Vec<Symbol> {
        let mut symbols = vec![];
        for (index, symbol) in self.symbols.iter().enumerate() {
            symbols.push(Symbol {
                grammar: self,
                index,
            });
        }
        symbols
    }

    pub fn terminals(&self) -> Vec<Symbol> {
        self.symbols().into_iter().filter(|symbol| symbol.is_terminal()).collect()
    }

    pub fn nonterminals(&self) -> Vec<Symbol> {
        self.symbols().into_iter().filter(|symbol| symbol.is_nonterminal()).collect()
    }

    pub fn symbol(&self, name: &str) -> Option<Symbol> {
        for (index, symbol_data) in self.symbols.iter().enumerate() {
            if &symbol_data.name == name {
                return Some(Symbol {
                    grammar: self,
                    index,
                });
            }

        }
        None
    }

    pub fn rules(&self) -> Vec<Rule> {
        let mut rules = vec![];
        for i in 0..self.rules.len() {
            rules.push(Rule(self, i));
        }
        rules
    }

    pub fn rules_for<'a>(&'a self, symbol: Symbol<'a>) -> Vec<Rule<'a>> {
        let mut rules = vec![];
        for (i, rule) in self.rules().into_iter().enumerate() {
            if rule.lhs() == symbol {
                rules.push(Rule(self, i));
            }
        }
        rules
    }

    pub fn nullables(&self) -> HashSet<Symbol> {
        let mut nullables = HashSet::new();

        loop {
            let mut dirty = false;

            for rule in self.rules() {
                if !nullables.contains(&rule.lhs()) {
                    if rule.rhs().iter().all(|symbol| nullables.contains(symbol)) {
                        nullables.insert(rule.lhs());
                        dirty = true;
                    }
                }
            }

            if !dirty {
                break;
            }
        }
        nullables
    }

    pub fn first_follows(&self) -> FirstFollows {
        let mut first_follows = FirstFollows::new(self);
        let nullables = self.nullables();

        for rule in self.rules() {
            for symbol in rule.rhs() {
                if symbol.is_terminal() {
                    first_follows.link(FFNode::First(rule.lhs()), FFNode::Terminal(symbol));
                } else {
                    first_follows.link(FFNode::First(rule.lhs()), FFNode::First(symbol));
                } 

                if !nullables.contains(&symbol) {
                    break;
                }
            }

            for (i, symbol) in rule.rhs().iter().copied().enumerate() {
                for j in i+1..rule.rhs().len() {
                    let follow = rule.rhs()[j];

                    if follow.is_terminal() {
                        first_follows.link(FFNode::Follow(symbol), FFNode::Terminal(follow));
                    } else {
                        first_follows.link(FFNode::Follow(symbol), FFNode::First(follow));
                    } 
                }
            }

            for symbol in rule.rhs().into_iter().rev() {
                if symbol.is_nonterminal() {
                    first_follows.link(FFNode::Follow(symbol), FFNode::Follow(rule.lhs()));
                }

                if !nullables.contains(&symbol) {
                    break;
                }
            }
        }

        first_follows
    }
}

pub struct FirstFollows<'a> {
    grammar: &'a Grammar,
    edges: HashMap<FFNode<'a>, HashSet<FFNode<'a>>>,
}

impl<'a> std::fmt::Debug for FirstFollows<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        for symbol in self.grammar.symbols() {
            if let Some(edges) = self.edges.get(&FFNode::First(symbol)) {
                writeln!(f, "FIRST({symbol:?}) = {edges:?}")?;
            }
        }
        for symbol in self.grammar.symbols() {
            if let Some(edges) = self.edges.get(&FFNode::Follow(symbol)) {
                writeln!(f, "FOLLOW({symbol:?}) = {edges:?}")?;
            }
        }
        Ok(())
    }
}

impl<'a> FirstFollows<'a> {
    fn new(grammar: &'a Grammar) -> Self {
        FirstFollows {
            grammar,
            edges: HashMap::new(),
        }
    }

    fn link(&mut self, from_node: FFNode<'a>, to_node: FFNode<'a>) {
        if !self.edges.contains_key(&from_node) {
            self.edges.insert(from_node, HashSet::new());
        }
        self.edges.get_mut(&from_node).unwrap().insert(to_node);
    }

    fn terminals_from(&self, from_node: FFNode<'a>) -> Vec<Symbol<'a>> {
        let mut visited = HashSet::new();
        let mut queue = vec![from_node];
        let mut terminals = vec![];

        while let Some(node) = queue.pop() {
            visited.insert(node);
            if let FFNode::Terminal(symbol) = node {
                terminals.push(symbol);
            } else if self.edges.contains_key(&node) {
                for next_node in &self.edges[&node] {
                    if !visited.contains(next_node) {
                        queue.push(*next_node);
                    }
                }
            }
        }
        terminals
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum FFNode<'a> {
    First(Symbol<'a>),
    Follow(Symbol<'a>),
    Terminal(Symbol<'a>),
}
