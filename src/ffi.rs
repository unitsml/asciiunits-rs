use crate::{Error, Parser};
use serde_json::{json, Value};
use std::ffi::{c_char, CString};
use std::sync::OnceLock;

static PARSER: OnceLock<Result<Parser, String>> = OnceLock::new();

fn parser() -> Result<&'static Parser, Error> {
    PARSER
        .get_or_init(|| Parser::new().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|message| Error::Artifact {
            message: message.clone(),
        })
}

fn response(result: Result<Value, Error>) -> *mut c_char {
    let reply = match result {
        Ok(value) => json!({"abi_version": 1, "ok": true, "value": value}),
        Err(error) => json!({"abi_version": 1, "ok": false, "error": error}),
    };
    CString::new(reply.to_string())
        .expect("JSON escapes NUL")
        .into_raw()
}

unsafe fn input_text<'a>(input: *const u8, length: usize) -> Result<&'a str, Error> {
    if input.is_null() || length > isize::MAX as usize {
        return Err(Error::InvalidArgument {
            message: "invalid input buffer".into(),
        });
    }
    let bytes = unsafe { std::slice::from_raw_parts(input, length) };
    std::str::from_utf8(bytes).map_err(|_| Error::InvalidArgument {
        message: "input is not UTF-8".into(),
    })
}

#[no_mangle]
pub extern "C" fn asciiunits_abi_version() -> u32 {
    1
}

/// Input is borrowed for the duration of this call. Release the returned
/// buffer exactly once with asciiunits_free_json_v1.
#[no_mangle]
pub unsafe extern "C" fn asciiunits_parse_json_v1(
    input: *const u8,
    length: usize,
    mode: u32,
) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        let text = unsafe { input_text(input, length) }?;
        match mode {
            0 => parser()?.parse_shape(text),
            1 => parser()?.parse(text),
            _ => Err(Error::InvalidArgument {
                message: "unknown parse mode".into(),
            }),
        }
    })
    .unwrap_or_else(|_| {
        Err(Error::Runtime {
            message: "native parser panicked".into(),
        })
    });
    response(result)
}

/// The typed model (`Parser::parse_model`) as its portable JSON:
/// `{"abi_version": 1, "ok": true, "model": {...}}`. A grammar refusal or
/// parser budget carries the same error payload as asciiunits_parse_json_v1;
/// a model the generated codecs refuse is `{"category": "model_error"}`.
/// Input is borrowed for the duration of this call. Release the returned
/// buffer exactly once with asciiunits_free_json_v1.
#[cfg(feature = "models")]
#[no_mangle]
pub unsafe extern "C" fn asciiunits_parse_model_json_v1(
    input: *const u8,
    length: usize,
) -> *mut c_char {
    use crate::typed::{to_portable_json, ModelError};
    let reply = std::panic::catch_unwind(|| {
        let text = match unsafe { input_text(input, length) } {
            Ok(text) => text,
            Err(error) => return json!({"abi_version": 1, "ok": false, "error": error}),
        };
        let model = parser()
            .map_err(ModelError::Parse)
            .and_then(|parser| parser.parse_model(text))
            .and_then(|model| to_portable_json(&model));
        match model {
            Ok(model) => json!({
                "abi_version": 1,
                "ok": true,
                "model": serde_json::from_str::<Value>(&model).expect("generated JSON codec"),
            }),
            Err(ModelError::Parse(error)) => {
                json!({"abi_version": 1, "ok": false, "error": error})
            }
            Err(error) => json!({
                "abi_version": 1,
                "ok": false,
                "error": {"category": "model_error", "message": error.to_string()},
            }),
        }
    })
    .unwrap_or_else(|_| {
        let error = Error::Runtime {
            message: "native parser panicked".into(),
        };
        json!({"abi_version": 1, "ok": false, "error": error})
    });
    CString::new(reply.to_string())
        .expect("JSON escapes NUL")
        .into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn asciiunits_free_json_v1(pointer: *mut c_char) {
    if !pointer.is_null() {
        drop(unsafe { CString::from_raw(pointer) });
    }
}
