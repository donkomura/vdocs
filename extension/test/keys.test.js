// Tests for keyEventToJson: the function that serialises a KeyboardEvent-like
// object into the exact JSON shape VimCore.on_key consumes.
//
// Guarantee taxonomy (mirrors the Rust side; see crates/vim_core/tests/parser_motion.rs):
//   G1. Mapping (input -> output is state-independent)
//   G5. Boundary safety (malformed input must not throw)

import { test } from "node:test";
import assert from "node:assert/strict";
import { keyEventToJson } from "../src/keys.js";

// G1. A plain alphabetic key with no modifiers must serialise to a JSON
// object with every modifier flag explicitly set to false. The Rust side
// uses #[serde(default)] for modifiers, but the JS side must still send
// them explicitly so that the wire format stays self-describing.
test("keyEventToJson serialises a bare key with explicit modifier flags", () => {
  const json = keyEventToJson({ key: "j", shiftKey: false, ctrlKey: false, altKey: false, metaKey: false });
  assert.equal(
    json,
    JSON.stringify({ key: "j", shift: false, ctrl: false, alt: false, meta: false }),
  );
});

// G1. The shift modifier must flow through so that VimCore can distinguish
// `g` (motion prefix) from `G` (DocEnd via shift). The Rust wasm_api test
// "modifier_flag_is_honored_across_json" pins the receiving side; this
// test pins the sending side.
test("keyEventToJson propagates shiftKey as shift:true", () => {
  const json = keyEventToJson({ key: "G", shiftKey: true, ctrlKey: false, altKey: false, metaKey: false });
  const parsed = JSON.parse(json);
  assert.equal(parsed.key, "G");
  assert.equal(parsed.shift, true);
});

// G1. ctrl / alt / meta propagate independently. The MVP parser does not
// act on them, but the boundary must already transport them so future
// bindings (e.g. Ctrl-d half-page-down) do not require a wire change.
test("keyEventToJson propagates ctrl, alt, meta independently", () => {
  const json = keyEventToJson({ key: "d", shiftKey: false, ctrlKey: true, altKey: false, metaKey: false });
  const parsed = JSON.parse(json);
  assert.equal(parsed.ctrl, true);
  assert.equal(parsed.alt, false);
  assert.equal(parsed.meta, false);
});

// G1. Special keys ("Escape", "ArrowDown", etc.) are passed through
// verbatim. The Rust parser dispatches on these exact strings, so any
// normalisation here would silently break mode transitions.
test("keyEventToJson passes special key names through unchanged", () => {
  const json = keyEventToJson({ key: "Escape", shiftKey: false, ctrlKey: false, altKey: false, metaKey: false });
  const parsed = JSON.parse(json);
  assert.equal(parsed.key, "Escape");
});

// G5. Missing modifier fields must not throw. DOM KeyboardEvent always
// carries the flags, but tests and synthetic events may omit them; the
// boundary must tolerate that and default to false.
test("keyEventToJson defaults missing modifier flags to false", () => {
  const json = keyEventToJson({ key: "j" });
  const parsed = JSON.parse(json);
  assert.equal(parsed.shift, false);
  assert.equal(parsed.ctrl, false);
  assert.equal(parsed.alt, false);
  assert.equal(parsed.meta, false);
});
