# AsciiUnits for Rust

`asciiunits::Parser::new()` verifies the edition-2 producer artifact.
`parse_shape` returns the exact capture tree and byte-indexed slices.
`parse` maps those captures to the producer's structural model JSON,
including the producer's fallback maps. The parser has separate syntax,
artifact, runtime and resource errors. `UPSTREAM.json` identifies the
artifact, mapper, model oracle and corpus revisions.

The crate also supplies the version-1 C ABI in `include/asciiunits.h`.
Build with `cargo build --locked`. Free each owned JSON result using
`asciiunits_free_json_v1`. The original input buffer is borrowed only
for the duration of the call.
