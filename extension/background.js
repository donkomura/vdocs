console.log("[vdocs] service worker initialized");

chrome.runtime.onInstalled.addListener(() => {
  console.log("[vdocs] extension installed");
  chrome.storage.local.set({ enabled: true });
});
