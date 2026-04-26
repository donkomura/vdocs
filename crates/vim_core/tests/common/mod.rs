//! 入力組み立ての雑務を吸収するヘルパ。ここには「保証」は無い。
//! テスト本体が語る保証がノイズなく読めるようにするためのもの。
//!
//! Rust の integration test は各ファイルが独立 crate として
//! コンパイルされるため、このモジュール内の一部関数はファイル
//! によっては未使用になる。dead_code を許可しておく。
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
