//! Parser の motion / count 系仕様。
//!
//! 各テストは冒頭コメントで「何を保証しているか」を G1〜G5 の
//! 区分つきで明示する:
//!   G1. 写像 (入力 → 出力が状態非依存)
//!   G2. 写像 + 状態遷移
//!   G3. 結果整合 (操作列の最終状態のみを保証)
//!   G4. 順序保証 (各ステップの出力順まで保証)
//!   G5. 境界安全 (不正入力で panic しない)

mod common;

use common::{feed, k, move_cmd, shifted};
use rstest::rstest;
use vim_core::command::Direction;
use vim_core::parser::Parser;

// G1. 単キー motion は状態に依存せず、キーと Direction の対応のみで
// Move(count=1) を返す。ケース列はこの「写像」の全域を示す。
#[rstest]
#[case("h", Direction::Left)]
#[case("j", Direction::Down)]
#[case("k", Direction::Up)]
#[case("l", Direction::Right)]
#[case("w", Direction::WordForward)]
#[case("b", Direction::WordBackward)]
#[case("e", Direction::WordEnd)]
fn single_key_maps_to_direction(#[case] key: &str, #[case] dir: Direction) {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k(key)), vec![move_cmd(dir, 1)]);
}

// G1 の特殊例: shift 修飾付きキー表現への写像。別入力形式だが
// 保証の形は同じ (状態非依存写像)。
#[rstest]
#[case("$", Direction::LineEnd)]
#[case("G", Direction::DocEnd)]
fn shifted_key_maps_to_direction(#[case] key: &str, #[case] dir: Direction) {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&shifted(key)), vec![move_cmd(dir, 1)]);
}

// G1 の例外条項: `0` は pending_count の有無で意味が変わる。
// (a) pending なし → LineStart
// (b) pending あり → count の桁として吸収
// この対比を 1 本で保証する (分けると仕様の対比が読めない)。
#[test]
fn zero_is_line_start_or_digit_depending_on_count_context() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("0")), vec![move_cmd(Direction::LineStart, 1)]);

    let mut p = Parser::new();
    let cmds = feed(&mut p, &[k("1"), k("0"), k("j")]);
    assert_eq!(cmds, vec![move_cmd(Direction::Down, 10)]);
}

// G4. count prefix は「前半 N-1 入力は空 Vec」「最後の 1 入力で
// Command が 1 回だけ出る」という順序付き保証。合計だけでなく
// 途中でリークしないことが本質なので、各ステップで検証する。
#[rstest]
#[case("5", "j", 5, Direction::Down)]
#[case("12", "l", 12, Direction::Right)]
#[case("10", "j", 10, Direction::Down)]
fn count_prefix_emits_exactly_once_at_final_key(
    #[case] digits: &str,
    #[case] motion: &str,
    #[case] expected_count: u32,
    #[case] dir: Direction,
) {
    let mut p = Parser::new();
    for ch in digits.chars() {
        let out = p.on_key(&k(&ch.to_string()));
        assert!(out.is_empty(), "digit `{ch}` must not emit any command yet");
    }
    assert_eq!(p.on_key(&k(motion)), vec![move_cmd(dir, expected_count)]);
}

// G3. count は 1 コマンドで消費される。2 回目以降は count=1 に戻る。
// 「最終状態」のみを検証する結果整合の保証。
#[test]
fn count_is_consumed_after_one_command() {
    let mut p = Parser::new();
    assert_eq!(
        feed(&mut p, &[k("5"), k("j")]),
        vec![move_cmd(Direction::Down, 5)]
    );
    assert_eq!(p.on_key(&k("j")), vec![move_cmd(Direction::Down, 1)]);
}

// G4. gg は「1 回目の g は空、2 回目の g で DocStart」という
// 順序付き保証を満たす。途中リーク (1 回目で何か出る) を禁じる。
#[test]
fn gg_emits_doc_start_only_on_second_g() {
    let mut p = Parser::new();
    assert!(p.on_key(&k("g")).is_empty());
    assert_eq!(p.on_key(&k("g")), vec![move_cmd(Direction::DocStart, 1)]);
}

// G3. g-prefix は非 g キーでキャンセル可能。キャンセル後の状態は
// フレッシュな Normal と等価 (後続 motion が通常通り動く) ことを
// 結果整合として保証する。
#[test]
fn g_prefix_is_cancelled_by_non_g_key() {
    let mut p = Parser::new();
    p.on_key(&k("g"));
    let _ = p.on_key(&k("x"));
    assert_eq!(p.on_key(&k("j")), vec![move_cmd(Direction::Down, 1)]);
}

// G5. 極端に大きい count を与えても panic せず u32 で飽和する。
// Wasm の trap を content script に漏らさないための境界安全保証。
#[test]
fn count_saturates_without_panic() {
    let mut p = Parser::new();
    let digits: Vec<_> = "99999999999999999999"
        .chars()
        .map(|c| k(&c.to_string()))
        .collect();
    for d in &digits {
        let _ = p.on_key(d);
    }
    let out = p.on_key(&k("j"));
    assert_eq!(out.len(), 1);
    match out[0] {
        vim_core::command::Command::Move { direction, count } => {
            assert_eq!(direction, Direction::Down);
            assert_eq!(count, u32::MAX);
        }
        _ => panic!("expected Move"),
    }
}
