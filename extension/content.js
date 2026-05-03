// Phase 2: load the VimCore Wasm module, wire it to document keydown,
// and log the commands VimCore emits. Acting on those commands (moving
// the cursor, editing text) is Phase 3.
//
// The content script runs in an isolated world and cannot use static
// `import`, so the Runner, the bootstrap, and the wasm glue are loaded
// via dynamic `import(chrome.runtime.getURL(...))`. Both `pkg/*` and
// `src/*` are declared under web_accessible_resources so these imports
// resolve.

console.log("[vdocs] content script loaded on Google Docs");

(async function () {
  try {
    const [{ default: init, VimCore }, { Runner }, { wireKeydown }] = await Promise.all([
      import(chrome.runtime.getURL("pkg/vim_core.js")),
      import(chrome.runtime.getURL("src/runner.js")),
      import(chrome.runtime.getURL("src/bootstrap.js")),
    ]);

    await init(chrome.runtime.getURL("pkg/vim_core_bg.wasm"));

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

    wireKeydown({
      document,
      runner,
      onCommands: (cmds) => {
        console.log("[vdocs] commands:", cmds, "mode:", runner.mode());
      },
    });

    console.log("[vdocs] VimCore initialised, mode:", runner.mode());
  } catch (error) {
    console.error("[vdocs] Failed to initialize:", error);
  }
})();
