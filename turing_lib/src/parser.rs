use std::collections::{HashMap, HashSet};

use crate::{
    machine::{HeadMovement, State, Symbol, Transition, TransitionSource, TuringMachine},
    tape::Tape,
};

struct Config {
    name: String,
    blank_symbol: char,
    head_start: usize,
}

pub fn parse_file(file_data: &str, tape: Tape) -> Result<TuringMachine, String> {
    let config: Config = parse_config(file_data)?;
    let (states, final_states, starting_state) = parse_states(file_data, config.blank_symbol)?;

    Ok(TuringMachine {
        name: config.name,
        blank_symbol: config.blank_symbol,

        states,
        final_states,

        head_idx: config.head_start,
        current_state: starting_state,
        tape,

        halted: false,
    })
}

fn parse_config(file_data: &str) -> Result<Config, String> {
    let mut config_lines = file_data.lines().skip_while(|&l| l != "config {").skip(1);
    let mut config_map = HashMap::new();

    while let Some(line) = config_lines.next() {
        match line.trim() {
            "}" => {
                break;
            }
            line => match line.split(": ").collect::<Vec<_>>()[..] {
                ["name", name] => {
                    if name.starts_with("\"") && name.ends_with("\"") {
                        config_map.insert(
                            "name",
                            name.trim_start_matches("\"")
                                .trim_end_matches("\"")
                                .to_string(),
                        );
                    } else {
                        return Err(
                                format!(
                                "Unexpected name value. It must be between double quotes (e.g. name: \"A name for the machine\")."));
                    }
                }
                ["blank_symbol", symbol] => match symbol.chars().collect::<Vec<_>>()[..] {
                    ['\'', symbol, '\''] => {
                        config_map.insert("blank_symbol", symbol.to_string());
                    }
                    _ => {
                        return Err(format!("Unexpected blank symbol. It must be a valid char between single quotes (e.g. blank_symbol: '_')."));
                    }
                },
                ["head_start", index] => {
                    config_map.insert("head_start", index.to_string());
                }
                _ => println!("Ignoring line \"{line}\""),
            },
        }
    }

    let name = config_map
        .remove("name")
        .expect("There was no name provided.");

    let blank_symbol = {
        let symbol = config_map
            .get("blank_symbol")
            .expect("There was no blank symbol provided.");
        symbol.chars().next().unwrap()
    };

    let head_start = {
        let index = config_map
            .get("head_start")
            .expect("There was no head start index provided.");

        index.parse().expect(&format!(
            "Invalid head start index provided (\"{index}\"). It must be a non negative integer."
        ))
    };

    Ok(Config {
        name,
        blank_symbol,
        head_start,
    })
}

fn parse_states(
    file_data: &str,
    blank_symbol: char,
) -> Result<(HashMap<String, State>, HashSet<String>, String), String> {
    struct ParsingState<'ps> {
        is_initial: bool,
        is_final: bool,
        name: &'ps str,
        transitions: HashMap<TransitionSource, Transition>,
    }

    let mut states = HashMap::new();
    let mut final_states = HashSet::new();
    let mut transition_states = HashSet::new(); // To check if all transitions are valid
    let mut initial_state_name = None;

    let mut state_lines = file_data.lines().skip_while(|&l| l != "states {").skip(1);

    let mut current_state: Option<ParsingState> = None;

    while let Some(line) = state_lines.next() {
        match line.trim() {
            "}" => {
                if current_state.is_some() {
                    let state = current_state.take().unwrap();

                    if state.is_initial {
                        if initial_state_name.is_some() {
                            return Err(format!("There was more than one initial state provided."));
                        }

                        initial_state_name = Some(state.name);
                    }

                    if state.is_final {
                        final_states.insert(state.name.to_string());
                    }

                    states.insert(
                        state.name.to_string(),
                        State::new(state.name.to_string(), state.transitions),
                    );
                } else {
                    break;
                }
            }
            line => match line
                .trim_end_matches("{")
                .split_whitespace()
                .collect::<Vec<_>>()[..]
            {
                ["state", state_name, "is", "initial", "and", "final"]
                | ["state", state_name, "is", "final", "and", "initial"] => {
                    current_state = Some(ParsingState {
                        is_initial: true,
                        is_final: true,
                        name: state_name,
                        transitions: HashMap::new(),
                    });
                }
                ["state", state_name, "is", "final"] => {
                    current_state = Some(ParsingState {
                        is_initial: false,
                        is_final: true,
                        name: state_name,
                        transitions: HashMap::new(),
                    });
                }
                ["state", state_name, "is", "initial"] => {
                    current_state = Some(ParsingState {
                        is_initial: true,
                        is_final: false,
                        name: state_name,
                        transitions: HashMap::new(),
                    });
                }
                ["state", state_name] => {
                    current_state = Some(ParsingState {
                        is_initial: false,
                        is_final: false,
                        name: state_name,
                        transitions: HashMap::new(),
                    });
                }
                _ => match line.trim().split(",").collect::<Vec<_>>()[..] {
                    [reading_symbol, writing_symbol, head_movement, new_state_name] => {
                        let reading_symbol = {
                            match &reading_symbol[..] {
                                "default" => TransitionSource::Default,
                                _ => {
                                    if reading_symbol.len() != 1 {
                                        return Err(format!(
                                            "Invalid reading symbol found at line \"{line}\""
                                        ));
                                    }

                                    let symbol = reading_symbol.chars().next().unwrap();

                                    if symbol == blank_symbol {
                                        TransitionSource::Blank
                                    } else {
                                        TransitionSource::Mark(symbol)
                                    }
                                }
                            }
                        };

                        let writing_symbol = {
                            match &writing_symbol[..] {
                                "default" => Symbol::Default,
                                _ => {
                                    if writing_symbol.len() != 1 {
                                        return Err(format!(
                                            "Invalid reading symbol found at line \"{line}\""
                                        ));
                                    }

                                    let symbol = writing_symbol.chars().next().unwrap();

                                    if symbol == blank_symbol {
                                        Symbol::Blank
                                    } else {
                                        Symbol::Mark(symbol)
                                    }
                                }
                            }
                        };

                        let head_movement = match head_movement {
                            "L" => HeadMovement::Left,
                            "R" => HeadMovement::Right,
                            "S" => HeadMovement::Stay,
                            _ => {
                                return Err(format!(
                                    "Unexpected head movement found at line \"{line}\""
                                ));
                            }
                        };

                        transition_states.insert(new_state_name);

                        if let Some(ref mut cur_state) = current_state {
                            cur_state.transitions.insert(
                                reading_symbol,
                                Transition::new(
                                    head_movement,
                                    writing_symbol,
                                    new_state_name.to_string(),
                                ),
                            );
                        } else {
                            return Err(format!(
                                "Unexpected transition declaration outside a state."
                            ));
                        }
                    }
                    _ => {
                        return Err(format!("Unexpected line \"{line}\"."));
                    }
                },
            },
        }
    }

    if !transition_states
        .iter()
        .all(|state_name| states.contains_key(*state_name))
    {
        return Err(
            "There are states that are transitioned into that are not defined.".to_string(),
        );
    }

    Ok((
        states,
        final_states,
        initial_state_name
            .expect("No initial state was provided.")
            .to_string(),
    ))
}
