use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use std::time::Duration;

use assert_matches::assert_matches;
use clap::Command;
use itertools::chain;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tempfile::TempDir;
use test_utils::get_absolute_path;
use validator::Validate;

use crate::command::{get_command_matches, update_config_map_by_command_args};
use crate::converters::deserialize_milliseconds_to_duration;
use crate::dumping::{
    append_sub_config_name,
    combine_config_map_and_pointers,
    ser_optional_param,
    ser_optional_sub_config,
    ser_param,
    ser_required_param,
    SerializeConfig,
};
use crate::loading::{
    load,
    load_and_process_config,
    split_pointers_map,
    split_values_and_types,
    update_config_map_by_pointers,
    update_optional_values,
};
use crate::{ConfigError, ParamPath, SerializationType, SerializedContent, SerializedParam};

lazy_static! {
    static ref CUSTOM_CONFIG_PATH: PathBuf =
        get_absolute_path("crates/papyrus_config/resources/custom_config_example.json");
}

#[derive(Clone, Copy, Default, Serialize, Deserialize, Debug, PartialEq, Validate)]
struct InnerConfig {
    #[validate(range(min = 0, max = 10))]
    o: usize,
}

impl SerializeConfig for InnerConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from([ser_param("o", &self.o, "This is o.")])
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Validate)]
struct OuterConfig {
    opt_elem: Option<usize>,
    opt_config: Option<InnerConfig>,
    #[validate]
    inner_config: InnerConfig,
}

impl SerializeConfig for OuterConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        chain!(
            ser_optional_param(&self.opt_elem, 1, "opt_elem", "This is elem."),
            ser_optional_sub_config(&self.opt_config, "opt_config"),
            append_sub_config_name(self.inner_config.dump(), "inner_config"),
        )
        .collect()
    }
}

#[test]
fn dump_and_load_config() {
    let some_outer_config = OuterConfig {
        opt_elem: Some(2),
        opt_config: Some(InnerConfig { o: 3 }),
        inner_config: InnerConfig { o: 4 },
    };
    let none_outer_config =
        OuterConfig { opt_elem: None, opt_config: None, inner_config: InnerConfig { o: 5 } };

    for outer_config in [some_outer_config, none_outer_config] {
        let (mut dumped, _) = split_values_and_types(outer_config.dump());
        update_optional_values(&mut dumped);
        let loaded_config = load::<OuterConfig>(&dumped).unwrap();
        assert_eq!(loaded_config, outer_config);
    }
}

#[test]
fn test_validation() {
    let outer_config =
        OuterConfig { opt_elem: None, opt_config: None, inner_config: InnerConfig { o: 20 } };
    assert!(outer_config.validate().is_err());
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct TypicalConfig {
    #[serde(deserialize_with = "deserialize_milliseconds_to_duration")]
    a: Duration,
    b: String,
    c: bool,
}

impl SerializeConfig for TypicalConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from([
            ser_param("a", &self.a.as_millis(), "This is a as milliseconds."),
            ser_param("b", &self.b, "This is b."),
            ser_param("c", &self.c, "This is c."),
        ])
    }
}

#[test]
fn test_update_dumped_config() {
    let command = Command::new("Testing");
    let dumped_config =
        TypicalConfig { a: Duration::from_secs(1), b: "bbb".to_owned(), c: false }.dump();
    let args = vec!["Testing", "--a", "1234", "--b", "15"];
    env::set_var("C", "true");
    let args: Vec<String> = args.into_iter().map(|s| s.to_owned()).collect();

    let arg_matches = get_command_matches(&dumped_config, command, args).unwrap();
    let (mut config_map, required_map) = split_values_and_types(dumped_config);
    update_config_map_by_command_args(&mut config_map, &required_map, &arg_matches).unwrap();

    assert_eq!(json!(1234), config_map["a"]);
    assert_eq!(json!("15"), config_map["b"]);
    assert_eq!(json!(true), config_map["c"]);

    let loaded_config: TypicalConfig = load(&config_map).unwrap();
    assert_eq!(Duration::from_millis(1234), loaded_config.a);
}

#[test]
fn test_pointers_flow() {
    let config_map = BTreeMap::from([
        ser_param("a1", &json!(5), "This is a."),
        ser_param("a2", &json!(5), "This is a."),
    ]);
    let pointers = vec![(
        ser_param("common_a", &json!(10), "This is common a"),
        vec!["a1".to_owned(), "a2".to_owned()],
    )];
    let stored_map = combine_config_map_and_pointers(config_map, &pointers).unwrap();
    assert_eq!(
        stored_map["a1"],
        json!(SerializedParam {
            description: "This is a.".to_owned(),
            content: SerializedContent::PointerTarget("common_a".to_owned()),
        })
    );
    assert_eq!(stored_map["a2"], stored_map["a1"]);
    assert_eq!(
        stored_map["common_a"],
        json!(SerializedParam {
            description: "This is common a".to_owned(),
            content: SerializedContent::DefaultValue(json!(10))
        })
    );

    let serialized = serde_json::to_string(&stored_map).unwrap();
    let loaded = serde_json::from_str(&serialized).unwrap();
    let (loaded_config_map, loaded_pointers_map) = split_pointers_map(loaded);
    let (mut config_map, _) = split_values_and_types(loaded_config_map);
    update_config_map_by_pointers(&mut config_map, &loaded_pointers_map).unwrap();
    assert_eq!(config_map["a1"], json!(10));
    assert_eq!(config_map["a1"], config_map["a2"]);
}

#[test]
fn test_replace_pointers() {
    let (mut config_map, _) =
        split_values_and_types(BTreeMap::from([ser_param("a", &json!(5), "This is a.")]));
    let pointers_map =
        BTreeMap::from([("b".to_owned(), "a".to_owned()), ("c".to_owned(), "a".to_owned())]);
    update_config_map_by_pointers(&mut config_map, &pointers_map).unwrap();
    assert_eq!(config_map["a"], config_map["b"]);
    assert_eq!(config_map["a"], config_map["c"]);

    let err = update_config_map_by_pointers(&mut BTreeMap::default(), &pointers_map).unwrap_err();
    assert_matches!(err, ConfigError::PointerTargetNotFound { .. });
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
struct CustomConfig {
    param_path: String,
}

impl SerializeConfig for CustomConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from([ser_param("param_path", &self.param_path, "This is param_path.")])
    }
}

// Loads param_path of CustomConfig from args.
fn load_param_path(args: Vec<&str>) -> String {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("config.json");
    CustomConfig { param_path: "default value".to_owned() }
        .dump_to_file(&vec![], file_path.to_str().unwrap())
        .unwrap();

    let loaded_config = load_and_process_config::<CustomConfig>(
        File::open(file_path).unwrap(),
        Command::new("Program"),
        args.into_iter().map(|s| s.to_owned()).collect(),
    )
    .unwrap();
    loaded_config.param_path
}

#[test]
fn test_load_default_config() {
    let args = vec!["Testing"];
    let param_path = load_param_path(args);
    assert_eq!(param_path, "default value");
}

#[test]
fn test_load_custom_config_file() {
    let args = vec!["Testing", "-f", CUSTOM_CONFIG_PATH.to_str().unwrap()];
    let param_path = load_param_path(args);
    assert_eq!(param_path, "custom value");
}

#[test]
fn test_load_custom_config_file_and_args() {
    let args = vec![
        "Testing",
        "--config_file",
        CUSTOM_CONFIG_PATH.to_str().unwrap(),
        "--param_path",
        "command value",
    ];
    let param_path = load_param_path(args);
    assert_eq!(param_path, "command value");
}

#[test]
fn test_load_many_custom_config_files() {
    let custom_config_path = CUSTOM_CONFIG_PATH.to_str().unwrap();
    let cli_config_param = format!("{custom_config_path},{custom_config_path}");
    let args = vec!["Testing", "-f", cli_config_param.as_str()];
    let param_path = load_param_path(args);
    assert_eq!(param_path, "custom value");
}

#[test]
fn serialization_precision() {
    let input =
        "{\"value\":244116128358498188146337218061232635775543270890529169229936851982759783745}";
    let serialized = serde_json::from_str::<serde_json::Value>(input).unwrap();
    let deserialized = serde_json::to_string(&serialized).unwrap();
    assert_eq!(input, deserialized);
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct RequiredConfig {
    param_path: String,
    num: usize,
}

impl SerializeConfig for RequiredConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from([
            ser_required_param("param_path", SerializationType::String, "This is param_path."),
            ser_param("num", &self.num, "This is num."),
        ])
    }
}

// Loads param_path of CustomConfig from args.
fn load_required_param_path(args: Vec<&str>) -> String {
    let dir = TempDir::new().unwrap();
    let file_path = dir.path().join("config.json");
    RequiredConfig { param_path: "default value".to_owned(), num: 3 }
        .dump_to_file(&vec![], file_path.to_str().unwrap())
        .unwrap();

    let loaded_config = load_and_process_config::<CustomConfig>(
        File::open(file_path).unwrap(),
        Command::new("Program"),
        args.into_iter().map(|s| s.to_owned()).collect(),
    )
    .unwrap();
    loaded_config.param_path
}

#[test]
fn test_negative_required_param() {
    let dumped_config = RequiredConfig { param_path: "0".to_owned(), num: 3 }.dump();
    let (config_map, _) = split_values_and_types(dumped_config);
    let err = load::<RequiredConfig>(&config_map).unwrap_err();
    assert_matches!(err, ConfigError::MissingParam { .. });
}

#[test]
fn test_required_param_from_command() {
    let args = vec!["Testing", "--param_path", "1234"];
    let param_path = load_required_param_path(args);
    assert_eq!(param_path, "1234");
}

#[test]
fn test_required_param_from_file() {
    let args = vec!["Testing", "--config_file", CUSTOM_CONFIG_PATH.to_str().unwrap()];
    let param_path = load_required_param_path(args);
    assert_eq!(param_path, "custom value");
}
