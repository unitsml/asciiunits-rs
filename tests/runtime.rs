use asciiunits::{Error, Parser, ARTIFACT};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// The shared corpus from unitsml/asciiunits-testsuite at the commit in
/// tests/corpus-pin.json. CI checks it out next to the crate; set
/// ASCIIUNITS_TESTSUITE to use another checkout. The bytes must match the pin.
fn pinned_corpus() -> Value {
    let pin: Value = serde_json::from_str(include_str!("corpus-pin.json")).unwrap();
    let root = std::env::var_os("ASCIIUNITS_TESTSUITE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(pin["checkout"].as_str().unwrap())
        });
    let path = root.join(pin["path"].as_str().unwrap());
    let bytes = std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "{}: {error}; check out {} at {}",
            path.display(),
            pin["repository"].as_str().unwrap(),
            pin["commit"].as_str().unwrap()
        )
    });
    let actual: String = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    assert_eq!(
        actual,
        pin["sha256"].as_str().unwrap(),
        "{}",
        path.display()
    );
    let corpus: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(corpus["artifact_checksum"], pin["artifact_checksum"]);
    corpus
}

#[test]
fn exact_source_raw_and_structural_models() {
    let parser = Parser::new().unwrap();
    let corpus = pinned_corpus();
    let artifact: Value = serde_json::from_str(ARTIFACT).unwrap();
    assert_eq!(corpus["artifact_checksum"], artifact["checksum"]);
    let rows = corpus["cases"].as_array().unwrap();
    assert_eq!(rows.len(), 143);
    let models: Vec<Value> = serde_json::from_str(include_str!("models.json")).unwrap();
    let mut checked = 0;
    for row in rows {
        let text = row["input"].as_str().unwrap();
        match row["kind"].as_str().unwrap() {
            "accept" => {
                assert_eq!(
                    parser.parse_shape(text).unwrap(),
                    row["parsanol_tree"],
                    "{text}"
                );
                assert_eq!(models[checked]["input"], text);
                assert_eq!(
                    parser.parse(text).unwrap(),
                    models[checked]["model"],
                    "{text}"
                );
                checked += 1;
            }
            "reject" => assert!(
                matches!(parser.parse_shape(text), Err(Error::Syntax { .. })),
                "{text}"
            ),
            other => panic!("unknown source outcome {other}"),
        }
    }
    assert_eq!(checked, models.len());
    assert_eq!(
        parser.parse("m").unwrap(),
        serde_json::json!({"unitName":"m"})
    );
}

#[test]
fn sealed_artifact_and_embedded_cases() {
    let parser = Parser::new().unwrap();
    assert!(parser.artifact_test_failures().is_empty());
    assert!(matches!(
        Parser::from_artifact(&format!("{ARTIFACT} ")),
        Err(Error::Artifact { .. })
    ));
}

#[test]
fn resource_abort_is_distinct_from_syntax_and_allows_reuse() {
    let parser = Parser::new().unwrap();
    assert!(parser.parse(&format!("{}m", "m/".repeat(128))).is_ok());
    let deep = format!("{}m", "m/".repeat(256));
    assert!(matches!(
        parser.parse_shape(&deep),
        Err(Error::RecursionLimit { .. })
    ));
    assert!(matches!(
        parser.parse(&deep),
        Err(Error::RecursionLimit { .. })
    ));
    assert!(matches!(
        parser.parse(""),
        Err(Error::Syntax { byte_offset: 0, .. })
    ));
    assert_eq!(
        parser.parse("m").unwrap(),
        serde_json::json!({"unitName":"m"})
    );
}
