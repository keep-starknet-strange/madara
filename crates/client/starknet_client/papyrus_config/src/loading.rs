//! Loads a configuration object, and set values for the fields in the following order of priority:
//! * Command line arguments.
//! * Environment variables (capital letters).
//! * Custom config files, separated by ',' (comma), from last to first.
//! * Default config file.

use std::collections::BTreeMap;
use std::fs::File;
use std::ops::IndexMut;
use std::path::PathBuf;

use clap::parser::Values;
use clap::Command;
use command::{get_command_matches, update_config_map_by_command_args};
use itertools::any;
use serde::Deserialize;
use serde_json::{json, Map, Value};

use crate::{command, ConfigError, ParamPath, SerializationType, SerializedContent, SerializedParam, IS_NONE_MARK};

/// Deserializes config from flatten JSON.
/// For an explanation of `for<'a> Deserialize<'a>` see
/// `<https://doc.rust-lang.org/nomicon/hrtb.html>`.
pub fn load<T: for<'a> Deserialize<'a>>(config_map: &BTreeMap<ParamPath, Value>) -> Result<T, ConfigError> {
    let mut nested_map = json!({});
    for (param_path, value) in config_map {
        let mut entry = &mut nested_map;
        for config_name in param_path.split('.') {
            entry = entry.index_mut(config_name);
        }
        *entry = value.clone();
    }
    Ok(serde_json::from_value(nested_map)?)
}

/// Deserializes a json config file, updates the values by the given arguments for the command, and
/// set values for the pointers.
pub fn load_and_process_config<T: for<'a> Deserialize<'a>>(
    default_config_file: File,
    command: Command,
    args: Vec<String>,
) -> Result<T, ConfigError> {
    let deserialized_default_config: Map<String, Value> = serde_json::from_reader(default_config_file)?;

    // Store the pointers separately from the default values. The pointers will receive a value
    // only at the end of the process.
    let (default_config_map, pointers_map) = split_pointers_map(deserialized_default_config);
    // Take param paths with corresponding descriptions, and get the matching arguments.
    let mut arg_matches = get_command_matches(&default_config_map, command, args)?;
    let (mut values_map, types_map) = split_values_and_types(default_config_map);
    // If the config_file arg is given, updates the values map according to this files.
    if let Some(custom_config_paths) = arg_matches.remove_many::<PathBuf>("config_file") {
        update_config_map_by_custom_configs(&mut values_map, &types_map, custom_config_paths)?;
    };
    // Updates the values map according to the args.
    update_config_map_by_command_args(&mut values_map, &types_map, &arg_matches)?;
    // Set values to the pointers.
    update_config_map_by_pointers(&mut values_map, &pointers_map)?;
    // Set values according to the is-none marks.
    update_optional_values(&mut values_map);
    // Build and return a Config object.
    load(&values_map)
}

// Separates a json map into config map of the raw values and pointers map.
pub(crate) fn split_pointers_map(
    json_map: Map<String, Value>,
) -> (BTreeMap<ParamPath, SerializedParam>, BTreeMap<ParamPath, ParamPath>) {
    let mut config_map: BTreeMap<String, SerializedParam> = BTreeMap::new();
    let mut pointers_map: BTreeMap<ParamPath, ParamPath> = BTreeMap::new();
    for (param_path, stored_param) in json_map {
        let Ok(ser_param) = serde_json::from_value::<SerializedParam>(stored_param.clone()) else {
            unreachable!("Invalid type in the json config map")
        };
        match ser_param.content {
            SerializedContent::PointerTarget(pointer_target) => {
                pointers_map.insert(param_path, pointer_target);
            }
            _ => {
                config_map.insert(param_path, ser_param);
            }
        };
    }
    (config_map, pointers_map)
}

// Removes the description from the config map, and splits the config map into default values and
// types of the default and required values.
// The types map includes required params, that do not have a value yet.
pub(crate) fn split_values_and_types(
    config_map: BTreeMap<ParamPath, SerializedParam>,
) -> (BTreeMap<ParamPath, Value>, BTreeMap<ParamPath, SerializationType>) {
    let mut values_map: BTreeMap<ParamPath, Value> = BTreeMap::new();
    let mut types_map: BTreeMap<ParamPath, SerializationType> = BTreeMap::new();
    for (param_path, serialized_param) in config_map {
        let Some(serialization_type) = serialized_param.content.get_serialization_type() else {
            continue;
        };
        types_map.insert(param_path.clone(), serialization_type);

        if let SerializedContent::DefaultValue(value) = serialized_param.content {
            values_map.insert(param_path, value);
        };
    }
    (values_map, types_map)
}

// Updates the config map by param path to value custom json files.
pub(crate) fn update_config_map_by_custom_configs(
    config_map: &mut BTreeMap<ParamPath, Value>,
    types_map: &BTreeMap<ParamPath, SerializationType>,
    custom_config_paths: Values<PathBuf>,
) -> Result<(), ConfigError> {
    for config_path in custom_config_paths {
        let file = std::fs::File::open(config_path)?;
        let custom_config: Map<String, Value> = serde_json::from_reader(file)?;
        for (param_path, json_value) in custom_config {
            update_config_map(config_map, types_map, param_path.as_str(), json_value)?;
        }
    }
    Ok(())
}

// Sets values in the config map to the params in the pointers map.
pub(crate) fn update_config_map_by_pointers(
    config_map: &mut BTreeMap<ParamPath, Value>,
    pointers_map: &BTreeMap<ParamPath, ParamPath>,
) -> Result<(), ConfigError> {
    for (param_path, target_param_path) in pointers_map {
        let Some(target_value) = config_map.get(target_param_path) else {
            return Err(ConfigError::PointerTargetNotFound { target_param: target_param_path.to_owned() });
        };
        config_map.insert(param_path.to_owned(), target_value.clone());
    }
    Ok(())
}

// Removes the none marks, and sets null for the params marked as None instead of the inner params.
pub(crate) fn update_optional_values(config_map: &mut BTreeMap<ParamPath, Value>) {
    let optional_params: Vec<_> = config_map
        .keys()
        .filter_map(|param_path| param_path.strip_suffix(&format!(".{IS_NONE_MARK}")))
        .map(|param_path| param_path.to_owned())
        .collect();
    let mut none_params = vec![];
    for optional_param in optional_params {
        let value = config_map.remove(&format!("{optional_param}.{IS_NONE_MARK}")).expect("Not found optional param");
        if value == json!(true) {
            none_params.push(optional_param);
        }
    }
    // Remove param paths that start with any None param.
    config_map.retain(|param_path, _| !any(&none_params, |none_param| param_path.starts_with(none_param)));
    for none_param in none_params {
        config_map.insert(none_param, Value::Null);
    }
}

pub(crate) fn update_config_map(
    config_map: &mut BTreeMap<ParamPath, Value>,
    types_map: &BTreeMap<ParamPath, SerializationType>,
    param_path: &str,
    new_value: Value,
) -> Result<(), ConfigError> {
    let Some(serialization_type) = types_map.get(param_path) else {
        return Err(ConfigError::ParamNotFound { param_path: param_path.to_string() });
    };
    let is_type_matched = match serialization_type {
        SerializationType::Number => new_value.is_number(),
        SerializationType::Boolean => new_value.is_boolean(),
        SerializationType::String => new_value.is_string(),
    };
    if !is_type_matched {
        return Err(ConfigError::ChangeRequiredParamType {
            param_path: param_path.to_string(),
            required: serialization_type.to_owned(),
            given: new_value,
        });
    }

    config_map.insert(param_path.to_owned(), new_value);
    Ok(())
}
