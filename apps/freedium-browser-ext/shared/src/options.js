const form = document.querySelector("#mirror-form");
const input = document.querySelector("#mirror");
const status = document.querySelector("#status");
const reset = document.querySelector("#reset");

function showStatus(message, kind) {
  status.textContent = message;
  if (kind) status.dataset.kind = kind;
  else delete status.dataset.kind;
}

async function load() {
  const stored = await chrome.storage.local.get({
    mirror: Freedium.DEFAULT_MIRROR,
  });
  try {
    input.value = Freedium.normalizeMirror(stored.mirror);
  } catch {
    input.value = Freedium.DEFAULT_MIRROR;
  }
}

form.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    const mirror = Freedium.normalizeMirror(input.value);
    await chrome.storage.local.set({ mirror });
    input.value = mirror;
    showStatus("Saved.");
  } catch (error) {
    showStatus(error.message, "error");
  }
});

reset.addEventListener("click", async () => {
  await chrome.storage.local.set({ mirror: Freedium.DEFAULT_MIRROR });
  input.value = Freedium.DEFAULT_MIRROR;
  showStatus("Using the default mirror.");
});

load();
