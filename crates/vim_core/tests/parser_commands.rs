//! Parser の mode / operator / edit 系仕様。
//!
//! 保証区分は parser_motion.rs の冒頭コメントを参照 (G1〜G5)。

mod common;

use common::{feed, k, move_cmd};
use rstest::rstest;
use vim_core::command::{Command, Direction, InsertAt, Target};
use vim_core::mode::Mode;
use vim_core::parser::Parser;

// G2. Insert モードのライフサイクル全体を 1 シナリオで保証する。
// (1) エントリキーで InsertModeEnter + mode==Insert
// (2) Insert 中は任意キーが空 Vec で素通し (コマンド出さない)
// (3) Escape で NormalModeEnter + mode==Normal
// 分解すると「素通し後 Escape で戻れない」等のシーケンス抜けを
// 捕まえられない。3 つのエントリキーは同じライフサイクル仕様な
// のでパラメタライズ。
#[rstest]
#[case("i", InsertAt::Before)]
#[case("a", InsertAt::After)]
#[case("o", InsertAt::NewlineBelow)]
fn insert_mode_lifecycle(#[case] enter_key: &str, #[case] at: InsertAt) {
    let mut p = Parser::new();

    let entered = p.on_key(&k(enter_key));
    assert_eq!(entered, vec![Command::InsertModeEnter { at }]);
    assert_eq!(p.mode(), Mode::Insert);

    assert!(
        p.on_key(&k("j")).is_empty(),
        "keys must pass through silently while in insert mode"
    );
    assert_eq!(p.mode(), Mode::Insert);

    let escaped = p.on_key(&k("Escape"));
    assert_eq!(escaped, vec![Command::NormalModeEnter]);
    assert_eq!(p.mode(), Mode::Normal);
}

// G1 + G4. x は count 付きで Delete Char を発行する。count=1 と
// count>1 を同一写像として扱う (count prefix が target.count に
// 載ることも含む)。
#[rstest]
#[case(&[], 1)]
#[case(&["3"], 3)]
#[case(&["1", "0"], 10)]
fn x_deletes_characters_with_count(#[case] prefix_keys: &[&str], #[case] expected: u32) {
    let mut p = Parser::new();
    let keys: Vec<_> = prefix_keys.iter().map(|s| k(s)).collect();
    let mut cmds = feed(&mut p, &keys);
    cmds.extend(p.on_key(&k("x")));
    assert_eq!(
        cmds,
        vec![Command::Delete {
            target: Target::Char { count: expected }
        }]
    );
}

// G4. Line-wise operator (dd/yy) の順序付き仕様:
// (a) operator 単独は空 Vec (待ち状態)
// (b) 同じ operator キーが続いたときだけ Line 対象の Command
// (c) count prefix は target.count に載る
// 3 条件は 1 体の仕様なので operator ごとに 1 シナリオで全部検証。
#[rstest]
#[case::delete("d", |c| Command::Delete { target: Target::Line { count: c } })]
#[case::yank("y", |c| Command::Yank { target: Target::Line { count: c } })]
fn linewise_operator_behaves_as_pair_with_count(
    #[case] op: &str,
    #[case] make: fn(u32) -> Command,
) {
    let mut p = Parser::new();
    assert!(
        p.on_key(&k(op)).is_empty(),
        "first operator key must be pending (empty output)"
    );
    assert_eq!(p.on_key(&k(op)), vec![make(1)]);

    let mut p = Parser::new();
    assert!(p.on_key(&k("3")).is_empty());
    assert!(p.on_key(&k(op)).is_empty());
    assert_eq!(p.on_key(&k(op)), vec![make(3)]);
}

// G3. operator の pending は Escape で確実にクリアされ、後続の
// motion が副作用なく動くことまで含めて「結果整合」を担保する。
#[test]
fn operator_pending_is_cancelled_by_escape() {
    let mut p = Parser::new();
    p.on_key(&k("d"));
    assert_eq!(p.on_key(&k("Escape")), vec![Command::NormalModeEnter]);
    assert_eq!(p.on_key(&k("j")), vec![move_cmd(Direction::Down, 1)]);
}

// G3. operator の後に不一致なキー (d → j 等) が来た場合、MVP では
// 「黙って捨てて状態をリセット」する契約。将来 operator+motion を
// サポートするときこのテストが赤くなって変更に気づける。
#[test]
fn operator_followed_by_non_repeat_is_silently_dropped() {
    let mut p = Parser::new();
    p.on_key(&k("d"));
    assert!(
        p.on_key(&k("j")).is_empty(),
        "d+j is not yet supported; output must be empty"
    );
    assert_eq!(p.on_key(&k("j")), vec![move_cmd(Direction::Down, 1)]);
}

// G1. p は count をサポートしない (MVP 仕様)。count prefix が
// あっても無視され、常に Paste After 1 回だけを返す。将来 count
// を導入するときこのテストが赤くなって気づける。
#[test]
fn paste_ignores_count_prefix_in_mvp() {
    let mut p = Parser::new();
    assert_eq!(
        p.on_key(&k("p")),
        vec![Command::Paste {
            at: InsertAt::After
        }]
    );

    let mut p = Parser::new();
    assert!(p.on_key(&k("5")).is_empty());
    assert_eq!(
        p.on_key(&k("p")),
        vec![Command::Paste {
            at: InsertAt::After
        }]
    );
}
