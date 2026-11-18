//! Typed AsciiUnits model: the parsanol capture tree read through the LutaML
//! models and JSON codecs generated from the Ruby model (src/generated).
//! Like `Asciiunits.parse_model` in Ruby, every node holds exactly the tree it
//! was built from; a capture the model does not declare is a codec error.
use crate::{
    generated::{json_codecs, models::Asciiunits_Model_Document},
    Error, Parser,
};
use serde_json::{json, Value};
use std::fmt;

#[derive(Debug)]
pub enum ModelError {
    Parse(Error),
    Codec(Box<lutaml_rs::contracts::ContractError>),
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(e) => write!(f, "{e}"),
            Self::Codec(e) => write!(f, "{e:?}"),
        }
    }
}

impl std::error::Error for ModelError {}

/// The Document JSON the generated reader takes for one capture tree. The
/// generated codecs read the portable-exact-json-v1 profile, which carries
/// exact integers (a slice's offset, length, line and column) as decimal
/// strings, so the tree's JSON numbers are written that way here.
pub fn document_json(tree: &Value) -> Value {
    json!({ "expression": portable(tree) })
}

fn portable(value: &Value) -> Value {
    match value {
        Value::Number(n) if n.is_u64() || n.is_i64() => Value::String(n.to_string()),
        Value::Array(items) => Value::Array(items.iter().map(portable).collect()),
        Value::Object(map) => {
            Value::Object(map.iter().map(|(k, v)| (k.clone(), portable(v))).collect())
        }
        _ => value.clone(),
    }
}

impl Parser {
    /// Parses `input` into the generated `Asciiunits::Model::Document`.
    pub fn parse_model(&self, input: &str) -> Result<Asciiunits_Model_Document, ModelError> {
        let tree = self.parse_shape(input).map_err(ModelError::Parse)?;
        json_codecs::read_Asciiunits_Model_Document_json(&document_json(&tree).to_string())
            .map_err(ModelError::Codec)
    }
}

/// The model's declared JSON (the Ruby model's JSON mappings).
pub fn to_portable_json(model: &Asciiunits_Model_Document) -> Result<String, ModelError> {
    json_codecs::write_Asciiunits_Model_Document_json(model).map_err(ModelError::Codec)
}
