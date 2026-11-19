//! Replays the pinned shared corpus through the generated typed model, as
//! scripts/replay_model.rb does for the Ruby model: for every accept row the
//! Document's declared JSON equals the recorded parsanol tree, and reading the
//! recorded tree writes the same JSON back; every reject row fails to parse.
#![cfg(feature = "models")]
mod support;

use asciiunits::{
    generated::json_codecs::read_Asciiunits_Model_Document_json,
    typed::{document_json, to_portable_json, ModelError},
    Parser,
};
use serde_json::Value;
use support::pinned_corpus;

#[test]
fn generated_models_replay_the_shared_corpus() {
    let parser = Parser::new().unwrap();
    let corpus = pinned_corpus();
    let (mut accepted, mut rejected) = (0, 0);
    for row in corpus["cases"].as_array().unwrap() {
        let text = row["input"].as_str().unwrap();
        match row["kind"].as_str().unwrap() {
            "accept" => {
                let expected = document_json(&row["parsanol_tree"]);
                let model = parser.parse_model(text).unwrap();
                model.validate().unwrap();
                let written: Value =
                    serde_json::from_str(&to_portable_json(&model).unwrap()).unwrap();
                assert_eq!(written, expected, "{text}");
                let recorded = read_Asciiunits_Model_Document_json(&expected.to_string()).unwrap();
                assert_eq!(recorded, model, "{text}");
                accepted += 1;
            }
            "reject" => {
                assert!(
                    matches!(parser.parse_model(text), Err(ModelError::Parse(_))),
                    "{text}"
                );
                rejected += 1;
            }
            other => panic!("unknown source outcome {other}"),
        }
    }
    assert_eq!((accepted, rejected), (92, 51));
}

#[test]
fn undeclared_capture_is_a_codec_error() {
    let tree = serde_json::json!({ "unit": { "value": "m", "offset": 0, "length": 1, "line": 1, "column": 1 }, "bogus": {} });
    assert!(read_Asciiunits_Model_Document_json(&document_json(&tree).to_string()).is_err());
}
