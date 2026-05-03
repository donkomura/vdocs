// Wire a keydown listener on the given document-like EventTarget so that
// every key is routed through the Runner, and non-empty command arrays
// are delivered to onCommands. Capture phase is required because Google
// Docs installs its own keydown handler that stops propagation on the
// editor iframe; listening in the bubble phase would drop every key.
export function wireKeydown({ document, runner, onCommands }) {
  document.addEventListener(
    "keydown",
    (e) => {
      const cmds = runner.handleKey(e);
      if (cmds.length === 0) return;
      try {
        onCommands(cmds);
      } catch (err) {
        // Absorb callback errors so one buggy consumer cannot break
        // subsequent key handling. Content script logs the error
        // through its own console.error; tests tolerate the swallow.
        console.error("[vdocs] onCommands threw", err);
      }
    },
    { capture: true },
  );
}
