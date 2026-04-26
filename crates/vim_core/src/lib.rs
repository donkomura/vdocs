use wasm_bindgen::prelude::*;

pub mod command;
pub mod mode;
pub mod parser;

use crate::parser::{Key, Parser};

#[wasm_bindgen]
pub struct VimCore {
    parser: Parser,
}

impl Default for VimCore {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl VimCore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
        }
    }

    pub fn on_key(&mut self, key_json: &str) -> String {
        let key: Key = match serde_json::from_str(key_json) {
            Ok(k) => k,
            Err(_) => return "[]".to_string(),
        };
        let cmds = self.parser.on_key(&key);
        serde_json::to_string(&cmds).unwrap_or_else(|_| "[]".to_string())
    }

    pub fn mode(&self) -> String {
        self.parser.mode().as_str().to_string()
    }
}
