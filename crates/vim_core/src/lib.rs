use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct VimCore {
    mode: String,
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
            mode: "normal".to_string(),
        }
    }

    pub fn on_key(&mut self, _key_json: &str) -> String {
        "[]".to_string()
    }

    pub fn mode(&self) -> String {
        self.mode.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vim_core_creation() {
        let core = VimCore::new();
        assert_eq!(core.mode(), "normal");
    }
}
