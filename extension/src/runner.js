import { keyEventToJson } from "./keys.js";

// Runner is the thin coordinator between the DOM and VimCore. It owns no
// DOM listeners itself — content.js wires those up and calls handleKey —
// so the runner can be unit-tested against a fake core without jsdom.
export class Runner {
  constructor({ core, isEnabled }) {
    this.core = core;
    this.isEnabled = isEnabled;
  }

  handleKey(e) {
    if (!this.isEnabled()) {
      return [];
    }
    const json = keyEventToJson(e);
    const raw = this.core.on_key(json);
    try {
      return JSON.parse(raw);
    } catch {
      // VimCore is contracted to return valid JSON (see
      // crates/vim_core/tests/wasm_api.rs). Absorb any regression here
      // rather than propagating — a throw would kill the content
      // script's keydown listener and silently break every subsequent
      // key, which is much worse than dropping one event.
      return [];
    }
  }

  mode() {
    return this.core.mode();
  }
}
