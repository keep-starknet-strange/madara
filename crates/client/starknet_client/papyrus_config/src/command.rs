use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{value_parser, Arg, ArgMatches, Command};
use serde_json::{json, Value};

use crate::loading::update_config_map;
use crate::{ConfigError, ParamPath, SerializationType, SerializedParam};

pub(crate) fn get_command_matches(
    config_map: &BTreeMap<ParamPath, SerializedParam>,
    command: Command,
    command_input: Vec<String>,
) -> Result<ArgMatches, ConfigError> {
    Ok(command.args(build_args_parser(config_map)).try_get_matches_from(command_input)?)
}

// Takes matched arguments from the command line interface and env variables and updates the config
// map.
// Supports usize, bool and String.
pub(crate) fn update_config_map_by_command_args(
    config_map: &mut BTreeMap<ParamPath, Value>,
    types_map: &BTreeMap<ParamPath, SerializationType>,
    arg_match: &ArgMatches,
) -> Result<(), ConfigError> {
    for param_path_id in arg_match.ids() {
        let param_path = param_path_id.as_str();
        let new_value = get_arg_by_type(types_map, arg_match, param_path)?;
        update_config_map(config_map, types_map, param_path, new_value)?;
    }
    Ok(())
}

// Builds the parser for the command line flags and env variables according to the types of the
// values in the config map.
fn build_args_parser(config_map: &BTreeMap<ParamPath, SerializedParam>) -> Vec<Arg> {
    let mut args_parser = vec![
        // Custom_config_file_path.
        Arg::new("config_file")
            .long("config_file")
            .short('f')
            .value_delimiter(',')
            .help("Optionally sets a config file to use")
            .value_parser(value_parser!(PathBuf)),
    ];

    for (param_path, serialized_param) in config_map.iter() {
        let Some(serialization_type) = serialized_param.content.get_serialization_type() else {
            continue; // Pointer target
        };
        let clap_parser = match serialization_type {
            SerializationType::Number => clap::value_parser!(usize).into(),
            SerializationType::Boolean => clap::value_parser!(bool),
            SerializationType::String => clap::value_parser!(String),
        };

        let arg = Arg::new(param_path)
            .long(param_path)
            .env(param_path.to_uppercase())
            .help(&serialized_param.description)
            .value_parser(clap_parser);
        args_parser.push(arg);
    }
    args_parser
}

// Converts clap arg_matches into json values.
fn get_arg_by_type(
    types_map: &BTreeMap<ParamPath, SerializationType>,
    arg_match: &ArgMatches,
    param_path: &str,
) -> Result<Value, ConfigError> {
    let serialization_type = types_map.get(param_path).expect("missing type");
    match serialization_type {
        SerializationType::Number => Ok(json!(arg_match.try_get_one::<usize>(param_path)?)),
        SerializationType::Boolean => Ok(json!(arg_match.try_get_one::<bool>(param_path)?)),
        SerializationType::String => Ok(json!(arg_match.try_get_one::<String>(param_path)?)),
    }
}
