use super::{parser, tape::Tape};
use std::{
    collections::{HashMap, HashSet},
    fs,
};

#[derive(Debug, Clone, Copy)]
pub enum HeadMovement {
    Left,
    Right,
    Stay,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub enum Symbol {
    Default, // Only used in Transition declarations (source symbol, new symbol)
    Mark(char),
    Blank,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum TransitionSource {
    Default,
    Mark(char),
    Blank,
}

pub struct State {
    pub name: String,
    pub transitions: HashMap<TransitionSource, Transition>,
}

#[derive(Debug)]
pub struct Transition {
    pub head_movement: HeadMovement,
    pub new_symbol: Symbol,
    pub new_state: String,
}

pub struct TuringMachine {
    pub(crate) name: String,
    pub(crate) blank_symbol: char,

    pub(crate) states: HashMap<String, State>,
    pub(crate) final_states: HashSet<String>,

    pub(crate) head_idx: usize,
    pub(crate) current_state: String,
    pub(crate) tape: Tape,

    pub(crate) halted: bool,
}

#[derive(Debug)]
pub enum TapeSide {
    Left,
    Right,
}

pub struct TickResult {
    pub written_different_symbol: bool,
    pub extended_tape_on_side: Option<TapeSide>,
    pub head_movement: HeadMovement,
}

impl TuringMachine {
    pub fn new_from_file(filename: &str, tape_data: &str) -> Result<TuringMachine, String> {
        let file_data = fs::read_to_string(filename)
            .map_err(|_| format!("Could not open the file \"{}\"", filename))?;

        let mut machine = parser::Parser::parse_file(&file_data, Tape(vec![]))?;
        let tape = Tape::parse(tape_data, machine.blank_symbol);
        machine.tape = tape;

        Ok(machine)
    }

    pub fn tick(&mut self) -> TickResult {
        if self.halted {
            return TickResult {
                written_different_symbol: false,
                extended_tape_on_side: None,
                head_movement: HeadMovement::Stay,
            };
        }

        let available_transitions = &self.states[&self.current_state].transitions;
        let current_symbol = &self.tape.read(self.head_idx);

        let transition = match current_symbol {
            Symbol::Default => available_transitions.get(&TransitionSource::Default),
            Symbol::Mark(c) => available_transitions.get(&TransitionSource::Mark(*c)),
            Symbol::Blank => available_transitions.get(&TransitionSource::Blank),
        };

        let transition = {
            if transition.is_some() {
                transition
            } else {
                available_transitions.get(&TransitionSource::Default)
            }
        };

        if let Some(transition) = transition {
            let new_symbol = if let Symbol::Default = transition.new_symbol {
                *current_symbol
            } else {
                transition.new_symbol
            };

            self.tape.write(self.head_idx, new_symbol);
            self.current_state = transition.new_state.clone();

            let extended_tape_on_side = match transition.head_movement {
                HeadMovement::Right => {
                    self.head_idx += 1;
                    if self.head_idx == self.tape.len() {
                        self.tape.extend_right();
                        Some(TapeSide::Right)
                    } else {
                        None
                    }
                }
                HeadMovement::Left => {
                    if self.head_idx == 0 {
                        self.tape.extend_left();
                        Some(TapeSide::Left)
                    } else {
                        self.head_idx -= 1;
                        None
                    }
                }
                HeadMovement::Stay => None,
            };

            return TickResult {
                written_different_symbol: new_symbol != *current_symbol,
                extended_tape_on_side,
                head_movement: transition.head_movement,
            };
        } else {
            println!("No transition available.");
            self.halted = true;
            return TickResult {
                written_different_symbol: false,
                extended_tape_on_side: None,
                head_movement: HeadMovement::Stay,
            };
            // Check for default behaviour. Else, halt.
            // println!("TODO")
        }
    }

    pub fn is_accepting(&self) -> bool {
        self.halted && self.final_states.contains(&self.current_state)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn blank_symbol(&self) -> char {
        self.blank_symbol
    }

    pub fn head_idx(&self) -> usize {
        self.head_idx
    }

    pub fn current_state_name(&self) -> &str {
        &self.current_state
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn tape(&self) -> &Tape {
        &self.tape
    }
}
