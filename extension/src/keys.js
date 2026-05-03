// Serialise a KeyboardEvent-like object into the exact JSON shape that
// VimCore.on_key expects. The Rust side uses #[serde(default)] for modifier
// flags, but the wire stays self-describing on purpose: JS always sends
// every flag explicitly so the transport is not ambiguous when debugging.
export function keyEventToJson(e) {
  return JSON.stringify({
    key: e.key,
    shift: Boolean(e.shiftKey),
    ctrl: Boolean(e.ctrlKey),
    alt: Boolean(e.altKey),
    meta: Boolean(e.metaKey),
  });
}
