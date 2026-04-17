const toggleButton = document.getElementById("toggle");
const statusDiv = document.getElementById("status");

chrome.storage.local.get(["enabled"], (result) => {
  if (chrome.runtime.lastError) {
    console.error('[vdocs] Storage error:', chrome.runtime.lastError);
    statusDiv.textContent = 'Status: Error';
    return;
  }
  const enabled = result.enabled !== false;
  updateStatus(enabled);
});

toggleButton.addEventListener("click", () => {
  chrome.storage.local.get(["enabled"], (result) => {
    if (chrome.runtime.lastError) {
      console.error('[vdocs] Storage error:', chrome.runtime.lastError);
      return;
    }
    const newState = !(result.enabled !== false);
    chrome.storage.local.set({ enabled: newState }, () => {
      if (chrome.runtime.lastError) {
        console.error('[vdocs] Storage error:', chrome.runtime.lastError);
        return;
      }
      updateStatus(newState);
    });
  });
});

function updateStatus(enabled) {
  statusDiv.textContent = `Status: ${enabled ? "ON" : "OFF"}`;
}
