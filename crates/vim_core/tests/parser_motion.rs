//! Parser specs for motions and count prefixes.
//!
//! Every test starts with a one-line comment naming the guarantee it
//! encodes, using the fixed taxonomy G1..G5:
//!   G1. Mapping (input -> output is state-independent)
//!   G2. Mapping + state transition
//!   G3. Eventual consistency (only the final state of a sequence is asserted)
//!   G4. Ordered output (per-step output sequence is asserted)
//!   G5. Boundary safety (malformed or extreme input must not panic)

mod common;

use common::{feed, k, move_cmd, shifted};
use rstest::rstest;
use vim_core::command::Direction;
use vim_core::parser::Parser;

// G1. A single-key motion is a state-independent mapping from the key to a
// Direction, producing Move(count=1). The case list enumerates the full
// domain of that mapping.
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

// G1, special case. Same state-independent mapping shape as above, but the
// input is expressed with a shift modifier instead of a raw key character.
#[rstest]
#[case("$", Direction::LineEnd)]
#[case("G", Direction::DocEnd)]
fn shifted_key_maps_to_direction(#[case] key: &str, #[case] dir: Direction) {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&shifted(key)), vec![move_cmd(dir, 1)]);
}

// G1, exception clause. `0` changes meaning depending on whether a count is
// pending:
//   (a) no pending count  -> LineStart
//   (b) pending count set -> absorbed as a digit
// The two cases are asserted together so the contrast stays visible;
// splitting them would hide the specification.
#[test]
fn zero_is_line_start_or_digit_depending_on_count_context() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("0")), vec![move_cmd(Direction::LineStart, 1)]);

    let mut p = Parser::new();
    let cmds = feed(&mut p, &[k("1"), k("0"), k("j")]);
    assert_eq!(cmds, vec![move_cmd(Direction::Down, 10)]);
}

// G4. A count prefix must produce exactly one Command on the final key and
// nothing on any earlier key. The essential property is no mid-sequence
// leak, so we assert the output at each step rather than just the total.
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

// G3. A count is consumed by one command; the next command falls back to
// count=1. Only the final state of the sequence is asserted.
#[test]
fn count_is_consumed_after_one_command() {
    let mut p = Parser::new();
    assert_eq!(
        feed(&mut p, &[k("5"), k("j")]),
        vec![move_cmd(Direction::Down, 5)]
    );
    assert_eq!(p.on_key(&k("j")), vec![move_cmd(Direction::Down, 1)]);
}

// G4. `gg` must emit nothing on the first `g` and DocStart on the second.
// Asserting the per-step output forbids any leak on the first key.
#[test]
fn gg_emits_doc_start_only_on_second_g() {
    let mut p = Parser::new();
    assert!(p.on_key(&k("g")).is_empty());
    assert_eq!(p.on_key(&k("g")), vec![move_cmd(Direction::DocStart, 1)]);
}

// G3. A pending `g` prefix is cancelled by any non-`g` key, and the parser
// returns to a state equivalent to a fresh Normal: a subsequent motion
// still works. We assert that equivalence as eventual consistency.
#[test]
fn g_prefix_is_cancelled_by_non_g_key() {
    let mut p = Parser::new();
    p.on_key(&k("g"));
    let _ = p.on_key(&k("x"));
    assert_eq!(p.on_key(&k("j")), vec![move_cmd(Direction::Down, 1)]);
}

// G5. An excessively large count must not panic; it saturates at u32::MAX.
// This keeps Wasm traps out of the content script.
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
