// Shared UI utilities and helpers

// State management functions
function markDirty() {
  if (dirty) return;
  dirty = true;
  window.__TAURI__.core.invoke("set_menu_enabled", { id: "save", enabled: true });
  window.__TAURI__.core.invoke("set_menu_enabled", { id: "reset", enabled: true });
}

function markClean() {
  dirty = false;
  window.__TAURI__.core.invoke("set_menu_enabled", { id: "save", enabled: false });
  window.__TAURI__.core.invoke("set_menu_enabled", { id: "reset", enabled: false });
}

function setupTabs() {
  for (const btn of document.querySelectorAll(".tab")) {
    btn.addEventListener("click", () => {
      if (btn.classList.contains("hidden")) return;
      document.querySelectorAll(".tab").forEach(b => b.classList.remove("active"));
      document.querySelectorAll(".tab-content").forEach(s => s.classList.remove("active"));
      btn.classList.add("active");
      document.getElementById("tab-" + btn.dataset.tab).classList.add("active");
    });
  }
}

function setRunnerTabEnabled(enabled) {
  const runnerTab = document.querySelector('.tab[data-tab="runner"]');
  const runnerPanel = document.getElementById("tab-runner");
  if (!runnerTab || !runnerPanel) return;

  if (enabled) {
    runnerTab.classList.remove("hidden");
    return;
  }

  runnerTab.classList.add("hidden");
  runnerPanel.classList.remove("active");

  if (runnerTab.classList.contains("active")) {
    runnerTab.classList.remove("active");
    const generalTab = document.querySelector('.tab[data-tab="general"]');
    const generalPanel = document.getElementById("tab-general");
    if (generalTab && generalPanel) {
      generalTab.classList.add("active");
      generalPanel.classList.add("active");
    }
  }
}

function addKvRow(container, key, value, keyPlaceholder, keyLabel) {
  const row = document.createElement("div");
  row.className = "kv-row";
  row.innerHTML = `
    <input type="text" class="kv-key" value="${escHtml(key)}" placeholder="${escHtml(keyPlaceholder)}" aria-label="${escHtml(keyLabel)}">
    <input type="text" class="kv-value" value="${escHtml(value)}" placeholder="Value">
    <button class="btn-remove" title="Remove">&times;</button>
  `;
  row.querySelector(".btn-remove").addEventListener("click", () => { row.remove(); markDirty(); });
  container.appendChild(row);
}

function textareaToList(id) {
  return document.getElementById(id).value
    .split("\n")
    .map(s => s.trim())
    .filter(s => s.length > 0);
}

function showStatus(msg, isError) {
  const el = document.getElementById("status-msg");
  el.textContent = msg;
  el.className = isError ? "error" : "";
  if (!isError) setTimeout(() => { el.textContent = ""; }, 3000);
}

function escHtml(s) {
  const d = document.createElement("div");
  d.textContent = s;
  return d.innerHTML;
}
