//! VimCore の wasm 境界契約 (wire format) を保証する統合テスト。
//!
//! このファイルは Parser の内部仕様ではなく、
//! JS 側が唯一依存する「JSON 文字列入力 → JSON 文字列出力」の
//! 契約を固定する。ここが壊れると Phase 2 以降の content.js が
//! 黙って動かなくなるため、内部リファクタで wire format が
//! 変わったことを即検出できるようにする。
//!
//! 保証区分は parser_motion.rs の冒頭コメントを参照 (G1〜G5)。

use rstest::rstest;
use serde_json::Value;
use vim_core::VimCore;

fn parse(s: &str) -> Value {
    serde_json::from_str(s).expect("VimCore must always return valid JSON")
}

// G1. wire format contract:
//   Move コマンドは必ず {"type":"Move","direction":<PascalCase>,"count":<u32>}
//   の 3 フィールド構造で出る。direction は PascalCase、count は数値。
//   JS 側はこの 3 フィールドにのみ依存して switch するため、
//   名前やケースが変わったらこのテストで即時検出する。
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

// G1. 全 Command バリアントが tag 付き object で出ることを、
// 代表入力でまとめて保証する。新しい Command を増やしたとき
// ケースを追加すれば追加漏れを検出できる。
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

// G1. NormalModeEnter は Escape 由来のコマンドで、追加
// フィールドなしの単独 tag として出る。Insert から Normal へ
// 戻るフローで JS 側が受け取る最初の合図なので確定しておく。
#[test]
fn normal_mode_enter_serializes_as_bare_tag() {
    let mut core = VimCore::new();
    core.on_key(r#"{"key":"i"}"#);
    let out = core.on_key(r#"{"key":"Escape"}"#);
    assert_eq!(parse(&out), serde_json::json!([{"type":"NormalModeEnter"}]));
}

// G1. shift 等のモディファイアフラグが JSON 経由で解釈される。
// JS 側で e.shiftKey を bool で詰めた入力が正しく DocEnd に
// 写像されることを境界越しで保証する。
#[test]
fn modifier_flag_is_honored_across_json() {
    let mut core = VimCore::new();
    let out = core.on_key(r#"{"key":"G","shift":true}"#);
    assert_eq!(
        parse(&out),
        serde_json::json!([{"type":"Move","direction":"DocEnd","count":1}])
    );
}

// G4. pending 状態が on_key 呼び出し間で保持される。JS 側は
// 「5 を送ると空配列が返る」「次の j で Move が出る」という
// 順序前提で実装するため、この保証が崩れると content.js の
// キー捕捉ロジックが破綻する。
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

// G2. mode() は on_key の副作用後の状態を文字列で返す。モード
// インジケータ表示の根拠。"normal" / "insert" のみを返す契約
// (他の値が混入しないこと) を固定する。
#[test]
fn mode_reflects_state_after_on_key_sequence() {
    let mut core = VimCore::new();
    assert_eq!(core.mode(), "normal");

    core.on_key(r#"{"key":"i"}"#);
    assert_eq!(core.mode(), "insert");

    core.on_key(r#"{"key":"Escape"}"#);
    assert_eq!(core.mode(), "normal");
}

// G5. 不正入力時の安全性保証: JSON が壊れていても、欠けていても
// panic せず "[]" を返す。Wasm で panic すると trap になり、
// その content script 以降のキー処理が止まるため、境界では必ず
// 吸収する。
#[rstest]
#[case("not json")]
#[case("")]
#[case(r#"{"no_key_field":"x"}"#)]
#[case(r#"{"key":123}"#)]
fn malformed_input_yields_empty_array_without_panic(#[case] input: &str) {
    let mut core = VimCore::new();
    assert_eq!(core.on_key(input), "[]");
}
