// Tests for Runner: the glue between the DOM and VimCore that owns the
// enabled flag, the VimCore instance, and the key -> commands pipeline.
//
// Guarantee taxonomy:
//   G1. Mapping
//   G2. Mapping + state transition
//   G3. Eventual consistency
//   G5. Boundary safety

import { test } from "node:test";
import assert from "node:assert/strict";
import { Runner } from "../src/runner.js";

function makeFakeCore(responses = new Map()) {
  const calls = [];
  return {
    calls,
    on_key(json) {
      calls.push(json);
      return responses.get(json) ?? "[]";
    },
    mode() {
      return "normal";
    },
  };
}

// G2. A key event in the enabled state must produce a JSON call into
// VimCore.on_key, and the returned command array must be parsed and
// returned to the caller. This pins the full request/response cycle.
test("Runner.handleKey forwards key to VimCore and returns parsed commands", () => {
  const expected = JSON.stringify({ key: "j", shift: false, ctrl: false, alt: false, meta: false });
  const core = makeFakeCore(new Map([[expected, '[{"type":"Move","direction":"Down","count":1}]']]));
  const runner = new Runner({ core, isEnabled: () => true });

  const cmds = runner.handleKey({ key: "j", shiftKey: false, ctrlKey: false, altKey: false, metaKey: false });

  assert.deepEqual(core.calls, [expected]);
  assert.deepEqual(cmds, [{ type: "Move", direction: "Down", count: 1 }]);
});

// G3. When the runner is disabled, no JSON is sent into VimCore and the
// result is an empty array. The eventual-consistency claim is "disabled
// == side-effect-free"; if any call leaks to the core, the popup toggle
// silently fails and keys double-fire.
test("Runner.handleKey is a no-op when disabled", () => {
  const core = makeFakeCore();
  const runner = new Runner({ core, isEnabled: () => false });

  const cmds = runner.handleKey({ key: "j" });

  assert.equal(core.calls.length, 0);
  assert.deepEqual(cmds, []);
});

// G2. The enabled flag is re-read on every key, not cached at
// construction. This is what makes the popup toggle take effect
// immediately without reloading the content script.
test("Runner.handleKey re-reads isEnabled on every call", () => {
  const core = makeFakeCore(new Map([
    [JSON.stringify({ key: "j", shift: false, ctrl: false, alt: false, meta: false }), "[]"],
  ]));
  let enabled = true;
  const runner = new Runner({ core, isEnabled: () => enabled });

  runner.handleKey({ key: "j" });
  assert.equal(core.calls.length, 1);

  enabled = false;
  runner.handleKey({ key: "j" });
  assert.equal(core.calls.length, 1, "disabling must stop further forwarding");

  enabled = true;
  runner.handleKey({ key: "j" });
  assert.equal(core.calls.length, 2, "re-enabling must resume forwarding");
});

// G5. If VimCore ever returns malformed JSON (it should not, but the
// wasm boundary is the kind of place where a refactor could regress
// this), the runner must absorb the error and return []. Propagating
// the exception would kill the content script's keydown listener.
test("Runner.handleKey returns empty array when core yields invalid JSON", () => {
  const badCore = {
    on_key: () => "not json",
    mode: () => "normal",
  };
  const runner = new Runner({ core: badCore, isEnabled: () => true });

  const cmds = runner.handleKey({ key: "j" });
  assert.deepEqual(cmds, []);
});

// G1. Runner.mode() is a thin delegate to VimCore.mode(). It exists so
// the content script and the popup can both read the mode without
// holding the core instance directly.
test("Runner.mode delegates to VimCore.mode", () => {
  const core = {
    on_key: () => "[]",
    mode: () => "insert",
  };
  const runner = new Runner({ core, isEnabled: () => true });
  assert.equal(runner.mode(), "insert");
});
