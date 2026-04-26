//! Integration tests that pin the VimCore wasm boundary
//! (JSON string in / JSON string out).
//!
//! The JS side depends only on the key names, types, and shape of the
//! JSON that VimCore returns. No matter how much we refactor the Rust
//! internals, if the externally visible shape changes the JS silently
//! breaks. This file exists to detect that regression immediately.
//!
//! See the header of parser_motion.rs for the G1..G5 guarantee taxonomy.

use rstest::rstest;
use serde_json::Value;
use vim_core::VimCore;

fn parse(s: &str) -> Value {
    serde_json::from_str(s).expect("VimCore must always return valid JSON")
}

// G1. Pins the JSON shape of a Move command. The JS side dispatches on
// {type, direction, count}, so any rename, case change (PascalCase), or
// type change in those fields must be caught immediately.
#[test]
fn wire_format_of_move_command_is_stable() {
    let mut core = VimCore::new();
    let out = core.on_key(r#"{"key":"j"}"#);
    let arr = parse(&out);
    assert_eq!(
        arr,
        serde_json::json!([{"type":"Move","direction":"Down","count":1}])
    );
}

// G1. Confirms that every Command variant is serialised as a "type"-tagged
// object, using one representative input per variant. Adding a new Command
// means adding a case here so a missing tag is detected.
#[rstest]
#[case(r#"{"key":"j"}"#, "Move")]
#[case(r#"{"key":"i"}"#, "InsertModeEnter")]
#[case(r#"{"key":"x"}"#, "Delete")]
#[case(r#"{"key":"p"}"#, "Paste")]
fn all_command_variants_serialize_with_type_tag(#[case] input: &str, #[case] expected_type: &str) {
    let mut core = VimCore::new();
    let out = core.on_key(input);
    let arr = parse(&out);
    let first = arr
        .as_array()
        .and_then(|a| a.first())
        .expect("non-empty array");
    assert_eq!(
        first.get("type").and_then(|v| v.as_str()),
        Some(expected_type)
    );
}

// G1. When Escape is pressed in Insert mode, the output is a single bare
// tag {"type":"NormalModeEnter"} with no extra fields. This is the only
// signal the JS side uses to detect returning to Normal, so the shape is
// pinned here.
#[test]
fn normal_mode_enter_serializes_as_bare_tag() {
    let mut core = VimCore::new();
    core.on_key(r#"{"key":"i"}"#);
    let out = core.on_key(r#"{"key":"Escape"}"#);
    assert_eq!(parse(&out), serde_json::json!([{"type":"NormalModeEnter"}]));
}

// G1. The shift flag embedded in the JSON input is interpreted across the
// boundary: shift+g must map to DocEnd. The JS side packs e.shiftKey as a
// bool, so if this path breaks every shifted key (G, $, ...) stops working.
#[test]
fn modifier_flag_is_honored_across_json() {
    let mut core = VimCore::new();
    let out = core.on_key(r#"{"key":"G","shift":true}"#);
    assert_eq!(
        parse(&out),
        serde_json::json!([{"type":"Move","direction":"DocEnd","count":1}])
    );
}

// G4. pending_count must survive across on_key calls. The JS side captures
// keys assuming "5 -> empty array" followed by "j -> Move(count=5)"; if
// that ordering is violated the count prefix stops working.
#[test]
fn pending_count_survives_across_on_key_calls() {
    let mut core = VimCore::new();
    assert_eq!(core.on_key(r#"{"key":"5"}"#), "[]");
    let out = core.on_key(r#"{"key":"j"}"#);
    assert_eq!(
        parse(&out),
        serde_json::json!([{"type":"Move","direction":"Down","count":5}])
    );
}

// G2. mode() returns the current mode as a string reflecting the state
// after on_key. The return value is restricted to "normal" / "insert";
// no other value may leak through. This API is the basis for the mode
// indicator, so the contract is pinned.
#[test]
fn mode_reflects_state_after_on_key_sequence() {
    let mut core = VimCore::new();
    assert_eq!(core.mode(), "normal");

    core.on_key(r#"{"key":"i"}"#);
    assert_eq!(core.mode(), "insert");

    core.on_key(r#"{"key":"Escape"}"#);
    assert_eq!(core.mode(), "normal");
}

// G5. Malformed input must not panic; the boundary returns "[]". A panic
// in Wasm is a trap that halts all subsequent key handling in the content
// script, so every error must be absorbed at the boundary.
#[rstest]
#[case("not json")]
#[case("")]
#[case(r#"{"no_key_field":"x"}"#)]
#[case(r#"{"key":123}"#)]
fn malformed_input_yields_empty_array_without_panic(#[case] input: &str) {
    let mut core = VimCore::new();
    assert_eq!(core.on_key(input), "[]");
}
