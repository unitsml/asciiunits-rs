use asciiunits::{Error, Parser, ARTIFACT};
use serde_json::Value;

#[test]
fn exact_source_raw_and_structural_models() {
    let parser = Parser::new().unwrap();
    let corpus: Value = serde_json::from_str(include_str!("corpus.json")).unwrap();
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
