const toggleButton = document.getElementById("toggle");
const statusDiv = document.getElementById("status");

chrome.storage.local.get(["enabled"], (result) => {
  const enabled = result.enabled !== false;
  updateStatus(enabled);
});

toggleButton.addEventListener("click", () => {
  chrome.storage.local.get(["enabled"], (result) => {
    const newState = !(result.enabled !== false);
    chrome.storage.local.set({ enabled: newState }, () => {
      updateStatus(newState);
    });
  });
});

function updateStatus(enabled) {
  statusDiv.textContent = `Status: ${enabled ? "ON" : "OFF"}`;
}
