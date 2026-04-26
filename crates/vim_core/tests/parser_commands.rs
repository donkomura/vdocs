//! Parser specs for modes, operators, and edit commands.
//!
//! See the header of parser_motion.rs for the G1..G5 guarantee taxonomy.

mod common;

use common::{feed, k, move_cmd};
use rstest::rstest;
use vim_core::command::{Command, Direction, InsertAt, Target};
use vim_core::mode::Mode;
use vim_core::parser::Parser;

// G2. The full Insert-mode lifecycle is asserted as one scenario:
//   (1) the entry key emits InsertModeEnter and mode becomes Insert
//   (2) while in Insert, any key passes through with empty output
//   (3) Escape emits NormalModeEnter and mode returns to Normal
// Splitting this into separate tests would miss sequencing bugs such as
// "pass-through works, but Escape no longer returns to Normal". The three
// entry keys share the same lifecycle spec, so they are parametrised.
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

// G1 + G4. `x` emits Delete Char with a count. count=1 and count>1 share
// the same mapping, and the count prefix must flow into target.count.
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

// G4. Ordered spec for line-wise operators (dd / yy):
//   (a) the operator alone must emit nothing (pending state)
//   (b) only a second press of the same operator key emits a Line command
//   (c) a count prefix must flow into target.count
// The three conditions form a single contract, so each operator is
// verified in one scenario that covers all of them.
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

// G3. A pending operator is reliably cleared by Escape, and a subsequent
// motion runs with no residual side effect. The eventual-consistency claim
// covers both the clear and the clean follow-up.
#[test]
fn operator_pending_is_cancelled_by_escape() {
    let mut p = Parser::new();
    p.on_key(&k("d"));
    assert_eq!(p.on_key(&k("Escape")), vec![Command::NormalModeEnter]);
    assert_eq!(p.on_key(&k("j")), vec![move_cmd(Direction::Down, 1)]);
}

// G3. When an operator is followed by a non-repeat key (e.g. d -> j), the
// MVP contract is "silently drop and reset state". When we later add
// operator+motion support, this test will turn red and flag the change.
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

// G1. `p` does not support counts in the MVP. A count prefix is ignored
// and `p` always emits a single Paste After. Adding count support later
// will turn this red and make the change visible.
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
