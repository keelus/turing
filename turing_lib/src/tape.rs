use std::fmt::Display;

use super::machine::Symbol;

#[derive(Debug, Clone)]
pub struct Tape(pub(crate) Vec<Symbol>);

impl Display for Tape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|symbol| match symbol {
                    Symbol::Mark(symbol) => format!("{}", symbol),
                    Symbol::Blank => "△".to_string(),
                    Symbol::Default => "".to_string(),
                })
                .collect::<String>()
        )
    }
}

impl Tape {
    pub fn parse(data: &str, blank_symbol: char) -> Tape {
        Tape(
            data.chars()
                .map(|c| {
                    if c == blank_symbol {
                        Symbol::Blank
                    } else {
                        Symbol::Mark(c)
                    }
                })
                .collect(),
        )
    }

    pub fn new(data: Vec<Symbol>) -> Self {
        Self(data)
    }

    pub fn read(&self, index: usize) -> Symbol {
        *self.0.get(index).unwrap()
    }

    pub fn write(&mut self, index: usize, symbol: Symbol) {
        self.0[index] = symbol
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn extend_right(&mut self) {
        self.0.push(Symbol::Blank);
    }

    pub fn extend_left(&mut self) {
        self.0.insert(0, Symbol::Blank);
    }

    pub fn get_content(&self) -> &[Symbol] {
        &self.0
    }
}
