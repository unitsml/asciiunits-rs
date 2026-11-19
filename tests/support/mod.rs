use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

/// The shared corpus from unitsml/asciiunits-testsuite at the commit in
/// tests/corpus-pin.json. CI checks it out next to the crate; set
/// ASCIIUNITS_TESTSUITE to use another checkout. The bytes must match the pin.
pub fn pinned_corpus() -> Value {
    let pin: Value = serde_json::from_str(include_str!("../corpus-pin.json")).unwrap();
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
