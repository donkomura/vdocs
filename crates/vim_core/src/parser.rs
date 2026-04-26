use serde::{Deserialize, Serialize};

use crate::command::{Command, Direction, InsertAt, Target};
use crate::mode::Mode;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Key {
    pub key: String,
    #[serde(default)]
    pub shift: bool,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub meta: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operator {
    Delete,
    Yank,
}

impl Operator {
    fn from_char(c: char) -> Option<Self> {
        match c {
            'd' => Some(Operator::Delete),
            'y' => Some(Operator::Yank),
            _ => None,
        }
    }

    fn apply_linewise(self, count: u32) -> Command {
        let target = Target::Line { count };
        match self {
            Operator::Delete => Command::Delete { target },
            Operator::Yank => Command::Yank { target },
        }
    }
}

#[derive(Debug, Default)]
pub struct Parser {
    mode: Mode,
    pending_count: Option<u32>,
    pending_operator: Option<Operator>,
    g_pending: bool,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn on_key(&mut self, key: &Key) -> Vec<Command> {
        if self.mode == Mode::Insert {
            return self.on_key_insert(key);
        }
        self.on_key_normal(key)
    }

    fn on_key_insert(&mut self, key: &Key) -> Vec<Command> {
        if key.key == "Escape" {
            self.mode = Mode::Normal;
            self.reset_pending();
            return vec![Command::NormalModeEnter];
        }
        Vec::new()
    }

    fn on_key_normal(&mut self, key: &Key) -> Vec<Command> {
        if key.key == "Escape" {
            self.reset_pending();
            return vec![Command::NormalModeEnter];
        }

        let Some(ch) = single_char(&key.key) else {
            return Vec::new();
        };

        if ch.is_ascii_digit() && (ch != '0' || self.pending_count.is_some()) {
            let digit = (ch as u32) - ('0' as u32);
            let current = self.pending_count.unwrap_or(0);
            self.pending_count = Some(current.saturating_mul(10).saturating_add(digit));
            return Vec::new();
        }

        if self.g_pending {
            self.g_pending = false;
            if ch == 'g' {
                let count = self.take_count();
                return vec![Command::Move {
                    direction: Direction::DocStart,
                    count,
                }];
            }
            self.reset_pending();
            return Vec::new();
        }

        if let Some(op) = self.pending_operator {
            if Operator::from_char(ch) == Some(op) {
                let count = self.take_count();
                self.pending_operator = None;
                return vec![op.apply_linewise(count)];
            }
            self.reset_pending();
            return Vec::new();
        }

        if ch == 'g' {
            self.g_pending = true;
            return Vec::new();
        }

        if let Some(op) = Operator::from_char(ch) {
            self.pending_operator = Some(op);
            return Vec::new();
        }

        self.dispatch_simple(ch, key)
    }

    fn dispatch_simple(&mut self, ch: char, key: &Key) -> Vec<Command> {
        let count = self.take_count();
        match ch {
            'h' => vec![move_cmd(Direction::Left, count)],
            'j' => vec![move_cmd(Direction::Down, count)],
            'k' => vec![move_cmd(Direction::Up, count)],
            'l' => vec![move_cmd(Direction::Right, count)],
            'w' => vec![move_cmd(Direction::WordForward, count)],
            'b' => vec![move_cmd(Direction::WordBackward, count)],
            'e' => vec![move_cmd(Direction::WordEnd, count)],
            '0' => vec![move_cmd(Direction::LineStart, count)],
            '$' => vec![move_cmd(Direction::LineEnd, count)],
            'G' => vec![move_cmd(Direction::DocEnd, count)],
            'i' => self.enter_insert(InsertAt::Before),
            'a' => self.enter_insert(InsertAt::After),
            'o' => self.enter_insert(InsertAt::NewlineBelow),
            'x' => vec![Command::Delete {
                target: Target::Char { count },
            }],
            'p' => vec![Command::Paste {
                at: InsertAt::After,
            }],
            _ => {
                let _ = key;
                Vec::new()
            }
        }
    }

    fn enter_insert(&mut self, at: InsertAt) -> Vec<Command> {
        self.mode = Mode::Insert;
        self.reset_pending();
        vec![Command::InsertModeEnter { at }]
    }

    fn take_count(&mut self) -> u32 {
        self.pending_count.take().unwrap_or(1)
    }

    fn reset_pending(&mut self) {
        self.pending_count = None;
        self.pending_operator = None;
        self.g_pending = false;
    }
}

fn move_cmd(direction: Direction, count: u32) -> Command {
    Command::Move { direction, count }
}

fn single_char(key: &str) -> Option<char> {
    let mut chars = key.chars();
    let first = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    Some(first)
}
