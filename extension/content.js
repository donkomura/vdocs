// Phase 2: route keydown into VimCore and log the emitted commands.
// Acting on those commands (moving the cursor, editing text) is Phase 3.
//
// Google Docs receives keystrokes via a hidden iframe
// (.docs-texteventtarget-iframe, src=about:blank). The top document's
// keydown listener does not see those keys. We therefore inject
// content.js into every frame (all_frames + match_about_blank in the
// manifest) and split responsibilities:
//
//   top frame   — owns VimCore; initialises the Wasm module; listens for
//                 forwarded key events from subframes.
//   sub frames  — no VimCore; forward keydown to the top via
//                 window.parent.postMessage.
//
// Initialising wasm-bindgen inside an about:blank frame fails because
// chrome.runtime.getURL resolves against the about:blank origin in a
// way that breaks the module's fetch. Keeping Wasm only in the top
// frame sidesteps that entirely.

const VDOCS_MSG = "vdocs:keydown";

function serialiseKey(e) {
  return {
    key: e.key,
    shiftKey: Boolean(e.shiftKey),
    ctrlKey: Boolean(e.ctrlKey),
    altKey: Boolean(e.altKey),
    metaKey: Boolean(e.metaKey),
  };
}

if (window.top !== window) {
  // Subframe: forward keydown to the top frame. No Wasm here.
  console.log("[vdocs] subframe content script loaded");
  document.addEventListener(
    "keydown",
    (e) => {
      try {
        window.parent.postMessage({ type: VDOCS_MSG, event: serialiseKey(e) }, "*");
      } catch (err) {
        console.error("[vdocs] failed to forward keydown", err);
      }
    },
    { capture: true },
  );
} else {
  console.log("[vdocs] content script loaded on Google Docs");

  (async function () {
    try {
      const [{ default: init, VimCore }, { Runner }, { wireKeydown }] = await Promise.all([
        import(chrome.runtime.getURL("pkg/vim_core.js")),
        import(chrome.runtime.getURL("src/runner.js")),
        import(chrome.runtime.getURL("src/bootstrap.js")),
      ]);

      await init({ module_or_path: chrome.runtime.getURL("pkg/vim_core_bg.wasm") });

      const core = new VimCore();

      // Cache the enabled flag in memory and keep it fresh via the
      // storage change listener. Reading chrome.storage on every keydown
      // would add latency to every keystroke.
      let enabled = true;
      const stored = await chrome.storage.local.get(["enabled"]);
      if (stored.enabled === false) enabled = false;
      chrome.storage.onChanged.addListener((changes, area) => {
        if (area === "local" && "enabled" in changes) {
          enabled = changes.enabled.newValue !== false;
        }
      });

      const runner = new Runner({ core, isEnabled: () => enabled });

      const onCommands = (cmds) => {
        console.log("[vdocs] commands:", cmds, "mode:", runner.mode());
      };

      wireKeydown({ document, runner, onCommands });

      // Key events from Google Docs' hidden text-event iframe reach the
      // subframe's document, not ours. The subframe branch above forwards
      // them here as postMessage payloads; we feed them back into Runner
      // so they share the same VimCore state as top-frame keys.
      window.addEventListener("message", (ev) => {
        if (!ev.data || ev.data.type !== VDOCS_MSG) return;
        const cmds = runner.handleKey(ev.data.event);
        if (cmds.length === 0) return;
        try {
          onCommands(cmds);
        } catch (err) {
          console.error("[vdocs] onCommands threw", err);
        }
      });

      console.log("[vdocs] VimCore initialised, mode:", runner.mode());
    } catch (error) {
      console.error("[vdocs] Failed to initialize:", error);
    }
  })();
}
