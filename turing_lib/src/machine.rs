use super::{parser, tape::Tape};
use std::{collections::HashMap, fs};

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
    pub name: String,
    pub blank_symbol: char,

    pub states: HashMap<String, State>,

    pub head_idx: usize,
    pub current_state: String,
    pub tape: Tape,

    pub halted: bool,
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
}

pub fn create_turing_machine() -> TuringMachine {
    let mut states = HashMap::new();
    states.insert("q0".to_string(), {
        let mut state_q0 = State {
            name: "q0".to_string(),
            transitions: HashMap::new(),
        };

        state_q0.transitions.insert(
            TransitionSource::Mark('0'),
            Transition {
                head_movement: HeadMovement::Left,
                new_symbol: Symbol::Mark('0'),
                new_state: "q0".to_string(),
            },
        );
        state_q0.transitions.insert(
            TransitionSource::Mark('1'),
            Transition {
                head_movement: HeadMovement::Left,
                new_symbol: Symbol::Mark('1'),
                new_state: "q0".to_string(),
            },
        );
        state_q0.transitions.insert(
            TransitionSource::Blank,
            Transition {
                head_movement: HeadMovement::Left,
                new_symbol: Symbol::Blank,
                new_state: "q0".to_string(),
            },
        );

        state_q0
    });

    TuringMachine {
        name: "A basic turing machine".to_string(),
        blank_symbol: '_',

        states,
        current_state: "q0".to_string(),

        head_idx: 5,
        tape: Tape::new(vec![
            Symbol::Blank,
            Symbol::Mark('1'),
            Symbol::Mark('1'),
            Symbol::Mark('0'),
            Symbol::Mark('1'),
            Symbol::Mark('0'),
            Symbol::Mark('0'),
            Symbol::Mark('0'),
            Symbol::Mark('1'),
            Symbol::Mark('1'),
            Symbol::Blank,
        ]),

        halted: false,
    }
}

pub fn create_turing_machine_2() -> TuringMachine {
    let mut states = HashMap::new();
    states.insert("q0".to_string(), {
        let mut state_q0 = State {
            name: "q0".to_string(),
            transitions: HashMap::new(),
        };

        state_q0.transitions.insert(
            TransitionSource::Default,
            Transition {
                head_movement: HeadMovement::Left,
                new_symbol: Symbol::Default,
                new_state: "q0".to_string(),
            },
        );

        state_q0
    });

    TuringMachine {
        name: "A basic turing machine".to_string(),
        blank_symbol: '_',

        states,
        current_state: "q0".to_string(),

        head_idx: 5,
        tape: Tape::new(vec![
            Symbol::Blank,
            Symbol::Mark('1'),
            Symbol::Mark('1'),
            Symbol::Mark('0'),
            Symbol::Mark('1'),
            Symbol::Mark('0'),
            Symbol::Mark('0'),
            Symbol::Mark('0'),
            Symbol::Mark('1'),
            Symbol::Mark('1'),
            Symbol::Blank,
        ]),

        halted: false,
    }
}
