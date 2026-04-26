use serde::{Deserialize, Serialize};

use crate::command::Command;
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

#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct Parser {
    mode: Mode,
    pending_count: Option<u32>,
    pending_operator: Option<char>,
    last_key: Option<char>,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn on_key(&mut self, _key: &Key) -> Vec<Command> {
        Vec::new()
    }
}
