console.log("[vdocs] content script loaded on Google Docs");

(async function() {
  console.log("[vdocs] attempting to load Wasm module...");

  try {
    const wasmUrl = chrome.runtime.getURL("pkg/vim_core.js");
    console.log("[vdocs] Wasm URL:", wasmUrl);

    console.log("[vdocs] Wasm module will be loaded in Phase 2");
  } catch (error) {
    console.error("[vdocs] Failed to initialize:", error);
  }
})();
