use vim_core::command::{Command, Direction, InsertAt, Target};
use vim_core::mode::Mode;
use vim_core::parser::{Key, Parser};

fn k(key: &str) -> Key {
    Key {
        key: key.to_string(),
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
    }
}

fn shift(key: &str) -> Key {
    Key {
        key: key.to_string(),
        shift: true,
        ctrl: false,
        alt: false,
        meta: false,
    }
}

fn feed(parser: &mut Parser, keys: &[Key]) -> Vec<Command> {
    let mut out = Vec::new();
    for key in keys {
        out.extend(parser.on_key(key));
    }
    out
}

fn move_cmd(direction: Direction, count: u32) -> Command {
    Command::Move { direction, count }
}

#[test]
fn h_moves_left_once() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("h")), vec![move_cmd(Direction::Left, 1)]);
}

#[test]
fn j_moves_down_once() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("j")), vec![move_cmd(Direction::Down, 1)]);
}

#[test]
fn k_moves_up_once() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("k")), vec![move_cmd(Direction::Up, 1)]);
}

#[test]
fn l_moves_right_once() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("l")), vec![move_cmd(Direction::Right, 1)]);
}

#[test]
fn count_prefix_5j_moves_down_five() {
    let mut p = Parser::new();
    let cmds = feed(&mut p, &[k("5"), k("j")]);
    assert_eq!(cmds, vec![move_cmd(Direction::Down, 5)]);
}

#[test]
fn multi_digit_count_12l_moves_right_twelve() {
    let mut p = Parser::new();
    let cmds = feed(&mut p, &[k("1"), k("2"), k("l")]);
    assert_eq!(cmds, vec![move_cmd(Direction::Right, 12)]);
}

#[test]
fn zero_without_pending_count_moves_to_line_start() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("0")), vec![move_cmd(Direction::LineStart, 1)]);
}

#[test]
fn zero_as_trailing_digit_in_count() {
    let mut p = Parser::new();
    let cmds = feed(&mut p, &[k("1"), k("0"), k("j")]);
    assert_eq!(cmds, vec![move_cmd(Direction::Down, 10)]);
}

#[test]
fn dollar_moves_to_line_end() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&shift("$")), vec![move_cmd(Direction::LineEnd, 1)]);
}

#[test]
fn gg_moves_to_doc_start() {
    let mut p = Parser::new();
    let first = p.on_key(&k("g"));
    assert!(first.is_empty(), "first g should wait for next key");
    let second = p.on_key(&k("g"));
    assert_eq!(second, vec![move_cmd(Direction::DocStart, 1)]);
}

#[test]
fn capital_g_moves_to_doc_end() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&shift("G")), vec![move_cmd(Direction::DocEnd, 1)]);
}

#[test]
fn w_moves_word_forward() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("w")), vec![move_cmd(Direction::WordForward, 1)]);
}

#[test]
fn b_moves_word_backward() {
    let mut p = Parser::new();
    assert_eq!(
        p.on_key(&k("b")),
        vec![move_cmd(Direction::WordBackward, 1)]
    );
}

#[test]
fn e_moves_word_end() {
    let mut p = Parser::new();
    assert_eq!(p.on_key(&k("e")), vec![move_cmd(Direction::WordEnd, 1)]);
}

#[test]
fn i_enters_insert_mode_before() {
    let mut p = Parser::new();
    let cmds = p.on_key(&k("i"));
    assert_eq!(
        cmds,
        vec![Command::InsertModeEnter {
            at: InsertAt::Before
        }]
    );
    assert_eq!(p.mode(), Mode::Insert);
}

#[test]
fn a_enters_insert_mode_after() {
    let mut p = Parser::new();
    let cmds = p.on_key(&k("a"));
    assert_eq!(
        cmds,
        vec![Command::InsertModeEnter {
            at: InsertAt::After
        }]
    );
    assert_eq!(p.mode(), Mode::Insert);
}

#[test]
fn o_opens_newline_below_and_enters_insert() {
    let mut p = Parser::new();
    let cmds = p.on_key(&k("o"));
    assert_eq!(
        cmds,
        vec![Command::InsertModeEnter {
            at: InsertAt::NewlineBelow
        }]
    );
    assert_eq!(p.mode(), Mode::Insert);
}

#[test]
fn escape_returns_to_normal_mode() {
    let mut p = Parser::new();
    p.on_key(&k("i"));
    assert_eq!(p.mode(), Mode::Insert);
    let cmds = p.on_key(&k("Escape"));
    assert_eq!(cmds, vec![Command::NormalModeEnter]);
    assert_eq!(p.mode(), Mode::Normal);
}

#[test]
fn insert_mode_passes_keys_through_without_emitting_commands() {
    let mut p = Parser::new();
    p.on_key(&k("i"));
    let cmds = p.on_key(&k("a"));
    assert!(
        cmds.is_empty(),
        "keys in insert mode should not emit commands"
    );
    assert_eq!(p.mode(), Mode::Insert);
}

#[test]
fn x_deletes_single_char() {
    let mut p = Parser::new();
    assert_eq!(
        p.on_key(&k("x")),
        vec![Command::Delete {
            target: Target::Char { count: 1 }
        }]
    );
}

#[test]
fn count_prefix_3x_deletes_three_chars() {
    let mut p = Parser::new();
    let cmds = feed(&mut p, &[k("3"), k("x")]);
    assert_eq!(
        cmds,
        vec![Command::Delete {
            target: Target::Char { count: 3 }
        }]
    );
}

#[test]
fn dd_deletes_line() {
    let mut p = Parser::new();
    let first = p.on_key(&k("d"));
    assert!(first.is_empty(), "first d should wait for operand");
    let second = p.on_key(&k("d"));
    assert_eq!(
        second,
        vec![Command::Delete {
            target: Target::Line { count: 1 }
        }]
    );
}

#[test]
fn count_prefix_3dd_deletes_three_lines() {
    let mut p = Parser::new();
    let cmds = feed(&mut p, &[k("3"), k("d"), k("d")]);
    assert_eq!(
        cmds,
        vec![Command::Delete {
            target: Target::Line { count: 3 }
        }]
    );
}

#[test]
fn yy_yanks_line() {
    let mut p = Parser::new();
    let first = p.on_key(&k("y"));
    assert!(first.is_empty(), "first y should wait for operand");
    let second = p.on_key(&k("y"));
    assert_eq!(
        second,
        vec![Command::Yank {
            target: Target::Line { count: 1 }
        }]
    );
}

#[test]
fn count_prefix_2yy_yanks_two_lines() {
    let mut p = Parser::new();
    let cmds = feed(&mut p, &[k("2"), k("y"), k("y")]);
    assert_eq!(
        cmds,
        vec![Command::Yank {
            target: Target::Line { count: 2 }
        }]
    );
}

#[test]
fn p_pastes_after() {
    let mut p = Parser::new();
    assert_eq!(
        p.on_key(&k("p")),
        vec![Command::Paste {
            at: InsertAt::After
        }]
    );
}

#[test]
fn operator_pending_is_cancelled_by_escape() {
    let mut p = Parser::new();
    p.on_key(&k("d"));
    let cmds = p.on_key(&k("Escape"));
    assert_eq!(cmds, vec![Command::NormalModeEnter]);
    let after = p.on_key(&k("j"));
    assert_eq!(after, vec![move_cmd(Direction::Down, 1)]);
}
