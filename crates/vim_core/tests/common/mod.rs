//! Helpers that absorb the boilerplate of building inputs. Nothing here
//! encodes a guarantee; the point is to keep each test's guarantee
//! readable without ceremony noise.
//!
//! Rust integration tests compile each file as a separate crate, so some
//! helpers in this module end up unused depending on which file pulls it
//! in. Allow dead code so that does not produce warnings.
#![allow(dead_code)]

use vim_core::command::{Command, Direction};
use vim_core::parser::{Key, Parser};

pub fn k(key: &str) -> Key {
    Key {
        key: key.to_string(),
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
    }
}

pub fn shifted(key: &str) -> Key {
    Key {
        key: key.to_string(),
        shift: true,
        ctrl: false,
        alt: false,
        meta: false,
    }
}

pub fn feed(parser: &mut Parser, keys: &[Key]) -> Vec<Command> {
    let mut out = Vec::new();
    for key in keys {
        out.extend(parser.on_key(key));
    }
    out
}

pub fn move_cmd(direction: Direction, count: u32) -> Command {
    Command::Move { direction, count }
}
