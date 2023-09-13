// config compiler to support no_coverage feature when running coverage in nightly mode within this
// crate
#![cfg_attr(coverage_nightly, feature(no_coverage))]
#![warn(missing_docs)]
//! Configuration utilities for a Starknet node.
//!
//! # Example
//!
//! ```
//! use std::collections::BTreeMap;
//! use std::fs::File;
//! use std::path::Path;
//!
//! use clap::Command;
//! use papyrus_config::dumping::{ser_param, SerializeConfig};
//! use papyrus_config::loading::load_and_process_config;
//! use papyrus_config::{ParamPath, SerializedParam};
//! use serde::{Deserialize, Serialize};
//! use tempfile::TempDir;
//!
//! #[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
//! struct ConfigExample {
//!     key: usize,
//! }
//!
//! impl SerializeConfig for ConfigExample {
//!     fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
//!         BTreeMap::from([ser_param("key", &self.key, "This is key description.")])
//!     }
//! }
//!
//! let dir = TempDir::new().unwrap();
//! let file_path = dir.path().join("config.json");
//! ConfigExample { key: 42 }.dump_to_file(&vec![], file_path.to_str().unwrap());
//! let file = File::open(file_path).unwrap();
//! let loaded_config = load_and_process_config::<ConfigExample>(
//!     file,
//!     Command::new("Program"),
//!     vec!["Program".to_owned(), "--key".to_owned(), "770".to_owned()],
//! )
//! .unwrap();
//! assert_eq!(loaded_config.key, 770);
//! ```

use clap::parser::MatchesError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) const IS_NONE_MARK: &str = "#is_none";

/// A nested path of a configuration parameter.
pub type ParamPath = String;
/// A description of a configuration parameter.
pub type Description = String;

#[cfg(test)]
mod config_test;

mod command;
pub mod converters;
pub mod dumping;
pub mod loading;
pub mod validators;

/// A serialized content of a configuration parameter.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SerializedContent {
    /// Serialized JSON default value.
    #[serde(rename = "value")]
    DefaultValue(Value),
    /// The target from which to take the JSON value of a configuration parameter.
    PointerTarget(ParamPath),
    /// Type of a required configuration parameter.
    RequiredType(SerializationType),
}

impl SerializedContent {
    fn get_serialization_type(&self) -> Option<SerializationType> {
        match self {
            SerializedContent::DefaultValue(value) => match value {
                Value::Number(_) => Some(SerializationType::Number),
                Value::Bool(_) => Some(SerializationType::Boolean),
                Value::String(_) => Some(SerializationType::String),
                _ => None,
            },
            SerializedContent::PointerTarget(_) => None,
            SerializedContent::RequiredType(ser_type) => Some(ser_type.clone()),
        }
    }
}

/// A description and serialized content of a configuration parameter.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SerializedParam {
    /// The description of the parameter.
    pub description: Description,
    /// The content of the parameter.
    #[serde(flatten)]
    pub content: SerializedContent,
}

/// A serialized type of a configuration parameter.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, strum_macros::Display)]
#[allow(missing_docs)]
pub enum SerializationType {
    Number,
    Boolean,
    String,
}

/// Errors at the configuration dumping and loading process.
#[allow(missing_docs)]
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error(transparent)]
    CommandInput(#[from] clap::error::Error),
    #[error(transparent)]
    MissingParam(#[from] serde_json::Error),
    #[error(transparent)]
    CommandMatches(#[from] MatchesError),
    #[error(transparent)]
    WriteDumpedConfig(#[from] std::io::Error),
    #[error("Insert a new param is not allowed.")]
    ParamNotFound { param_path: String },
    #[error("{target_param} is not found.")]
    PointerTargetNotFound { target_param: String },
    #[error("{pointing_param} is not found.")]
    PointerSourceNotFound { pointing_param: String },
    #[error("Changing {param_path} from required type {required} to {given} is not allowed.")]
    ChangeRequiredParamType { param_path: String, required: SerializationType, given: Value },
}
