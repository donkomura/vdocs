// Tests for wireKeydown: the glue that attaches a keydown listener to a
// document-like target and drives Runner.handleKey. Uses the stdlib
// EventTarget so we can run under plain node:test without jsdom.
//
// Guarantee taxonomy:
//   G2. Mapping + state transition
//   G3. Eventual consistency
//   G5. Boundary safety

import { test } from "node:test";
import assert from "node:assert/strict";
import { wireKeydown } from "../src/bootstrap.js";
import { Runner } from "../src/runner.js";

function makeCore(response = "[]") {
  return {
    on_key: () => response,
    mode: () => "normal",
  };
}

// Node does not expose the DOM KeyboardEvent constructor. For these
// tests we only need an EventTarget-dispatchable object that carries a
// `key` field; the Runner only reads `key` / `shiftKey` / `ctrlKey` /
// `altKey` / `metaKey` on the event, so a plain Event with extra props
// is a valid stand-in.
function keydownEvent(init) {
  const e = new Event("keydown");
  return Object.assign(e, {
    key: init.key,
    shiftKey: Boolean(init.shiftKey),
    ctrlKey: Boolean(init.ctrlKey),
    altKey: Boolean(init.altKey),
    metaKey: Boolean(init.metaKey),
  });
}

// A minimal document-like EventTarget that exposes addEventListener with
// the same signature as the real DOM. It is enough for wireKeydown's
// contract; we are not testing the actual browser.
function makeDoc() {
  const target = new EventTarget();
  // EventTarget already exposes addEventListener; wrap to record calls.
  const originalAdd = target.addEventListener.bind(target);
  const calls = [];
  target.addEventListener = (type, listener, options) => {
    calls.push({ type, options });
    originalAdd(type, listener, options);
  };
  target._addCalls = calls;
  return target;
}

// G2. A keydown KeyboardEvent on the document must produce exactly one
// call to onCommands with the parsed command array. This is the
// end-to-end contract the content script depends on.
test("wireKeydown delivers parsed commands to the callback on keydown", () => {
  const core = {
    on_key: () => '[{"type":"Move","direction":"Down","count":1}]',
    mode: () => "normal",
  };
  const runner = new Runner({ core, isEnabled: () => true });
  const doc = makeDoc();
  const received = [];

  wireKeydown({ document: doc, runner, onCommands: (cmds) => received.push(cmds) });

  doc.dispatchEvent(keydownEvent({ key: "j" }));

  assert.deepEqual(received, [[{ type: "Move", direction: "Down", count: 1 }]]);
});

// G3. When VimCore returns an empty command array (e.g. first key of a
// pending sequence like `g`), onCommands must not fire. The content
// script logs commands conditionally, and a spurious empty-array log
// would flood the console.
test("wireKeydown skips onCommands when the command array is empty", () => {
  const runner = new Runner({ core: makeCore("[]"), isEnabled: () => true });
  const doc = makeDoc();
  const received = [];

  wireKeydown({ document: doc, runner, onCommands: (cmds) => received.push(cmds) });
  doc.dispatchEvent(keydownEvent({ key: "g" }));

  assert.deepEqual(received, []);
});

// G2. wireKeydown attaches the listener in the capture phase. Google Docs
// installs its own keydown handler that calls stopPropagation, so we
// must intercept before that path. Losing the capture flag here would
// silently break every key in the real extension.
test("wireKeydown registers the listener with capture: true", () => {
  const runner = new Runner({ core: makeCore(), isEnabled: () => true });
  const doc = makeDoc();

  wireKeydown({ document: doc, runner, onCommands: () => {} });

  assert.equal(doc._addCalls.length, 1);
  const { type, options } = doc._addCalls[0];
  assert.equal(type, "keydown");
  // Either a boolean true or an options bag with capture: true is
  // acceptable; the semantics are identical.
  if (typeof options === "boolean") {
    assert.equal(options, true);
  } else {
    assert.equal(options?.capture, true);
  }
});

// G5. If the onCommands callback throws, the keydown listener must not
// propagate the exception. An uncaught throw inside the listener would
// be visible in the console but would not stop further key handling in
// a real browser — however, we still want the content script to stay
// robust, and we own the callback boundary.
test("wireKeydown swallows errors thrown by onCommands", () => {
  const core = {
    on_key: () => '[{"type":"Move","direction":"Down","count":1}]',
    mode: () => "normal",
  };
  const runner = new Runner({ core, isEnabled: () => true });
  const doc = makeDoc();

  wireKeydown({
    document: doc,
    runner,
    onCommands: () => {
      throw new Error("boom");
    },
  });

  // If wireKeydown did not guard, dispatchEvent would propagate and
  // fail the test with the Error thrown above.
  assert.doesNotThrow(() => {
    doc.dispatchEvent(keydownEvent({ key: "j" }));
  });
});
