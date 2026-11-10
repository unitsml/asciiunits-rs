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
        if input.is_null() || length > isize::MAX as usize {
            return Err(Error::InvalidArgument {
                message: "invalid input buffer".into(),
            });
        }
        let bytes = unsafe { std::slice::from_raw_parts(input, length) };
        let text = std::str::from_utf8(bytes).map_err(|_| Error::InvalidArgument {
            message: "input is not UTF-8".into(),
        })?;
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

#[no_mangle]
pub unsafe extern "C" fn asciiunits_free_json_v1(pointer: *mut c_char) {
    if !pointer.is_null() {
        drop(unsafe { CString::from_raw(pointer) });
    }
}
