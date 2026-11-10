//! Edition-2 AsciiUnits structural parsing through Parsanol.
mod ffi;
mod mapper;

use parsanol::{PargArtifact, PargError};
use serde::Serialize;
use serde_json::Value;

pub const ARTIFACT: &str = include_str!("../artifacts/unitsml.json");
pub const ARTIFACT_SHA256: &str =
    "139d91337a0944d60f36bc144a205625349f4b3b8b7ff1a0fba3c61e70c57804";
const ENTRY: &str = "expression";

#[derive(Debug, Serialize)]
#[serde(tag = "category", rename_all = "snake_case")]
pub enum Error {
    Syntax {
        byte_offset: usize,
        line: usize,
        column: usize,
        expected: Vec<String>,
    },
    RecursionLimit {
        depth: usize,
        max_depth: usize,
    },
    CallDepthLimit {
        depth: usize,
        max_depth: usize,
    },
    Artifact {
        message: String,
    },
    Runtime {
        message: String,
    },
    InvalidArgument {
        message: String,
    },
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax {
                byte_offset,
                expected,
                ..
            } => write!(
                formatter,
                "syntax refusal at byte {byte_offset}: {expected:?}"
            ),
            Self::RecursionLimit { depth, max_depth }
            | Self::CallDepthLimit { depth, max_depth } => {
                write!(formatter, "parser depth {depth} exceeds budget {max_depth}")
            }
            Self::Artifact { message }
            | Self::Runtime { message }
            | Self::InvalidArgument { message } => formatter.write_str(message),
        }
    }
}
impl std::error::Error for Error {}

impl From<PargError> for Error {
    fn from(error: PargError) -> Self {
        match error {
            PargError::ParseWire {
                offset,
                line,
                column,
                expected,
                ..
            } => Self::Syntax {
                byte_offset: offset,
                line,
                column,
                expected,
            },
            PargError::RecursionLimitExceeded { depth, max_depth } => {
                Self::RecursionLimit { depth, max_depth }
            }
            PargError::CallDepthLimitExceeded { depth, max_depth } => {
                Self::CallDepthLimit { depth, max_depth }
            }
            other => Self::Runtime {
                message: other.to_string(),
            },
        }
    }
}

pub struct Parser {
    artifact: PargArtifact,
}

impl Parser {
    pub fn new() -> Result<Self, Error> {
        Self::from_artifact(ARTIFACT)
    }

    /// Accept only the pinned producer artifact, including its exact bytes.
    pub fn from_artifact(source: &str) -> Result<Self, Error> {
        if source.as_bytes() != ARTIFACT.as_bytes() {
            return Err(Error::Artifact {
                message: "AsciiUnits artifact does not match the pinned bytes".into(),
            });
        }
        let artifact = PargArtifact::from_json(source).map_err(|error| Error::Artifact {
            message: error.to_string(),
        })?;
        Ok(Self { artifact })
    }

    pub fn parse_shape(&self, input: &str) -> Result<Value, Error> {
        self.artifact.parse_shape(ENTRY, input).map_err(Error::from)
    }

    /// Structural model in the producer's JSON field format, including raw fallback maps.
    pub fn parse(&self, input: &str) -> Result<Value, Error> {
        self.parse_shape(input).map(mapper::map_tree)
    }

    pub fn artifact_test_failures(&self) -> Vec<String> {
        self.artifact.run_tests()
    }
}
