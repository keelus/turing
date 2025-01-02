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

pub fn parse_file(file_lines: &[&str], tape: Tape) -> Result<TuringMachine, String> {
    let config: Config = parse_config(file_lines)?;
    let (states, final_states, starting_state) = parse_states(file_lines, config.blank_symbol)?;

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

fn parse_config(file_data: &[&str]) -> Result<Config, String> {
    let config_lines = file_data.iter().skip_while(|&&l| l != "config {").skip(1);
    let mut config_map = HashMap::new();

    for line in config_lines {
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
                                "[turing_lib] Error while parsing configuration. Unexpected name value. It must be between double quotes (e.g. name: \"A name for the machine\").".to_string());
                    }
                }
                ["blank_symbol", symbol] => match symbol.chars().collect::<Vec<_>>()[..] {
                    ['\'', symbol, '\''] => {
                        config_map.insert("blank_symbol", symbol.to_string());
                    }
                    _ => {
                        return Err("[turing_lib] Error while parsing configuration. Unexpected blank symbol. It must be a valid char between single quotes (e.g. blank_symbol: '_').".to_string());
                    }
                },
                ["head_start", index] => {
                    config_map.insert("head_start", index.to_string());
                }
                _ => println!("Ignoring line \"{line}\""),
            },
        }
    }

    if config_map.is_empty() {
        return Err(
            "[turing_lib] Error while parsing configuration. There was no configuration provided."
                .to_string(),
        );
    }

    let name = config_map.remove("name").ok_or_else(|| {
        "[turing_lib] Error while parsing configuration. There was no name provided.".to_string()
    })?;

    let blank_symbol = {
        let symbol = config_map
            .get("blank_symbol")
            .ok_or_else(|| "[turing_lib] Error while parsing configuration. There was no blank symbol provided.".to_string())?;
        symbol.chars().next().unwrap()
    };

    let head_start = {
        let index = config_map
            .get("head_start")
            .ok_or_else(|| "[turing_lib] Error while parsing configuration. There was no head start index provided.".to_string())?;

        index.parse().map_err(|_| format!("[turing_lib] Error while parsing configuration. Invalid head start index provided (\"{index}\"). It must be a non negative integer."))?
    };

    Ok(Config {
        name,
        blank_symbol,
        head_start,
    })
}

fn parse_states(
    file_data: &[&str],
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

    let state_lines = file_data.iter().skip_while(|&&l| l != "states {").skip(1);

    let mut current_state: Option<ParsingState> = None;

    let mut append_state = |state: ParsingState<'_>| -> Result<_, String> {
        if state.is_initial {
            if initial_state_name.is_some() {
                return Err("[turing_lib] Error while parsing states. There was more than one initial state provided.".to_string());
            }

            initial_state_name = Some(state.name.to_string());
        }

        if state.is_final {
            final_states.insert(state.name.to_string());
        }

        states.insert(
            state.name.to_string(),
            State::new(state.name.to_string(), state.transitions),
        );
        Ok(())
    };

    for line in state_lines {
        match line.trim() {
            "}" => {
                if current_state.is_some() {
                    append_state(current_state.take().unwrap())?;
                } else {
                    break;
                }
            }
            line => {
                let (state_def_line, is_empty_state) = if line.trim().ends_with("}") {
                    (
                        line.trim().trim_end_matches("}").trim_end_matches("{"),
                        true,
                    )
                } else {
                    (line.trim().trim_end_matches("{"), false)
                };

                match state_def_line.split_whitespace().collect::<Vec<_>>()[..] {
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
                                                "[turing_lib] Error while parsing states. Invalid reading symbol found at line \"{line}\""
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
                                                "[turing_lib] Error while parsing states. Invalid reading symbol found at line \"{line}\""
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
                                        "[turing_lib] Error while parsing states. Unexpected head movement found at line \"{line}\""
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
                                return Err("[turing_lib] Error while parsing states. Unexpected transition declaration outside a state."
                                    .to_string());
                            }
                        }
                        _ => {
                            return Err(format!("[turing_lib] Error while parsing states. Unexpected line \"{line}\"."));
                        }
                    },
                }

                if is_empty_state {
                    append_state(current_state.take().unwrap())?;
                }
            }
        }
    }

    if !transition_states
        .iter()
        .all(|state_name| states.contains_key(*state_name))
    {
        return Err(
            "[turing_lib] Error while parsing states. There are states that are transitioned into that are not defined.".to_string(),
        );
    }

    Ok((
        states,
        final_states,
        initial_state_name.ok_or_else(|| {
            "[turing_lib] Error while parsing states. No initial state was provided.".to_string()
        })?,
    ))
}
