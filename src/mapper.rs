//! Structural mapper for the AsciiUnits edition-2 capture contract.
//! The producer's mapper and model at ff147f2 define these JSON fields.
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

fn normalize(node: Value) -> Value {
    match node {
        Value::Array(values) => Value::Array(values.into_iter().map(normalize).collect()),
        Value::Object(fields) => Value::Object(
            fields
                .into_iter()
                .filter(|(name, _)| {
                    !matches!(name.as_str(), "end_guard" | "lparen" | "rparen" | "hyphen")
                })
                .map(|(name, child)| (name, normalize(child)))
                .collect(),
        ),
        value => value,
    }
}

pub(crate) fn map_tree(tree: Value) -> Value {
    map(normalize(tree)).0
}

fn prefix(name: Value) -> Value {
    json!({"prefixName": name, "onlyInstance": false})
}

fn unit(name: Value, power: Option<Value>, pref: Option<Value>) -> Value {
    let mut fields = Map::new();
    fields.insert("unitName".into(), name);
    if let Some(value) = power {
        fields.insert("powerNumerator".into(), value);
    }
    if let Some(value) = pref {
        fields.insert("prefix".into(), value);
    }
    Value::Object(fields)
}

fn dimension(name: Value, power: Option<Value>) -> Value {
    let mut fields = Map::new();
    fields.insert("dimensionName".into(), name);
    if let Some(value) = power {
        fields.insert("powerNumerator".into(), value);
    }
    Value::Object(fields)
}

fn fence(open: Value, value: Value, close: Value) -> Value {
    json!({"open": open, "value": value, "close": close})
}

fn formula<const N: usize>(value: [Value; N]) -> Value {
    json!({"value": Value::Array(Vec::from(value))})
}

fn map(node: Value) -> (Value, bool) {
    let Value::Object(fields) = node else {
        return (node, false);
    };
    let mut fenced = BTreeSet::new();
    let mut mapped = Map::new();
    for (key, child) in fields {
        let (value, is_fenced) = map(child);
        if is_fenced {
            fenced.insert(key.clone());
        }
        mapped.insert(key, value);
    }
    let mut keys: Vec<&str> = mapped.keys().map(String::as_str).collect();
    keys.sort_unstable();
    let get = |name: &str| mapped[name].clone();
    let text = |name: &str| match &mapped[name] {
        Value::Object(slice) => slice.get("value").cloned().unwrap_or(Value::Null),
        Value::String(s) => Value::String(s.clone()),
        scalar => Value::String(scalar.to_string()),
    };
    let exponent = || Some(get("power_numerator"));
    let pref = || Some(prefix(text("prefix")));
    let extender = || json!({"text":text("extender")});
    let number = || json!({"value":text("integer")});
    let set = |name: &str| {
        if fenced.contains(name) {
            get(name)
        } else {
            fence(json!("("), get(name), json!(")"))
        }
    };
    let value = match keys.as_slice() {
        ["dimension"] => dimension(text("dimension"), None),
        ["dimension", "extender", "power_numerator", "sequence"] => formula([
            dimension(text("dimension"), exponent()),
            extender(),
            get("sequence"),
        ]),
        ["dimension", "extender", "sequence"] => formula([
            dimension(text("dimension"), None),
            extender(),
            get("sequence"),
        ]),
        ["dimension", "power_numerator"] => dimension(text("dimension"), exponent()),
        ["extender", "first_set", "second_set"] => {
            formula([get("first_set"), extender(), get("second_set")])
        }
        ["extender", "implicit_extended", "sequence"] => {
            formula([get("implicit_extended"), extender(), get("sequence")])
        }
        ["extender", "int_exp", "sequence"] => {
            formula([get("int_exp"), extender(), get("sequence")])
        }
        ["extender", "power_numerator", "sequence", "unit"] => formula([
            unit(text("unit"), exponent(), None),
            extender(),
            get("sequence"),
        ]),
        ["extender", "power_numerator", "prefix", "sequence", "unit"] => formula([
            unit(text("unit"), exponent(), pref()),
            extender(),
            get("sequence"),
        ]),
        ["extender", "prefix", "sequence", "unit"] => formula([
            unit(text("unit"), None, pref()),
            extender(),
            get("sequence"),
        ]),
        ["extender", "sequence", "unit"] => {
            formula([unit(text("unit"), None, None), extender(), get("sequence")])
        }
        ["first_int_exp_set", "second_int_exp_set"] => {
            formula([set("first_int_exp_set"), set("second_int_exp_set")])
        }
        ["first_int_exp_set", "second_set"] => {
            formula([set("first_int_exp_set"), get("second_set")])
        }
        ["first_set", "second_int_exp_set"] => {
            formula([get("first_set"), set("second_int_exp_set")])
        }
        ["first_set", "second_set"] => formula([get("first_set"), get("second_set")]),
        ["implicit_extended"] => {
            return (
                get("implicit_extended"),
                fenced.contains("implicit_extended"),
            )
        }
        ["int_exp"] => return (get("int_exp"), fenced.contains("int_exp")),
        ["int_exp", "sequence"] => formula([get("int_exp"), get("sequence")]),
        ["integer"] => number(),
        ["close_paren", "integer", "open_paren"] => {
            return (
                fence(text("open_paren"), number(), text("close_paren")),
                true,
            )
        }
        ["close_paren", "int_exp", "open_paren"] => {
            return (
                fence(text("open_paren"), get("int_exp"), text("close_paren")),
                true,
            )
        }
        ["power_numerator", "unit"] => unit(text("unit"), exponent(), None),
        ["power_numerator", "prefix", "unit"] => unit(text("unit"), exponent(), pref()),
        ["prefix"] => prefix(text("prefix")),
        ["prefix", "unit"] => unit(text("unit"), None, pref()),
        ["sqrt"] => json!({"value":get("sqrt")}),
        ["unit"] => unit(text("unit"), None, None),
        _ => Value::Object(mapped),
    };
    (value, false)
}
