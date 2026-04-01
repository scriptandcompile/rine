// rine-config frontend — Tauri IPC via window.__TAURI__
const { invoke } = window.__TAURI__.core;

let exePath = null;
let config = null;

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

async function init() {
  // Ask the backend for the exe path passed as a CLI argument.
  try {
    exePath = await invoke("get_exe_path");
  } catch (_) {
    // ignore — will show the welcome screen
  }

  if (exePath) {
    await loadConfig(exePath);
  } else {
    document.getElementById("no-exe").classList.remove("hidden");
    document.getElementById("editor").classList.add("hidden");
  }

  setupTabs();
  setupButtons();
}

// ---------------------------------------------------------------------------
// Config loading & saving
// ---------------------------------------------------------------------------

async function loadConfig(path) {
  exePath = path;
  try {
    config = await invoke("get_config", { exePath: path });
    const cfgPath = await invoke("get_config_path", { exePath: path });

    document.getElementById("no-exe").classList.add("hidden");
    document.getElementById("editor").classList.remove("hidden");

    document.getElementById("exe-path-display").textContent = path;
    document.getElementById("config-path-display").textContent = cfgPath;

    await populateVersions();
    populateForm();
  } catch (err) {
    showStatus("Failed to load config: " + err, true);
  }
}

async function populateVersions() {
  const versions = await invoke("get_windows_versions");
  const sel = document.getElementById("win-version");
  sel.innerHTML = "";
  for (const v of versions) {
    const opt = document.createElement("option");
    opt.value = JSON.stringify(v.value);
    opt.textContent = v.label;
    sel.appendChild(opt);
  }
}

function populateForm() {
  if (!config) return;

  // Windows version
  const sel = document.getElementById("win-version");
  const verStr = JSON.stringify(config.windows_version);
  for (const opt of sel.options) {
    if (opt.value === verStr) { opt.selected = true; break; }
  }

  // Filesystem
  document.getElementById("default-root").value = config.filesystem.default_root || "";
  document.getElementById("case-insensitive").checked = config.filesystem.case_insensitive;

  // Drives
  const driveList = document.getElementById("drive-list");
  driveList.innerHTML = "";
  for (const [letter, path] of Object.entries(config.filesystem.drives || {})) {
    addKvRow(driveList, letter, path, "A-Z", "Drive letter");
  }

  // DLL
  document.getElementById("search-order").value = (config.dll.search_order || []).join("\n");
  document.getElementById("force-stub").value = (config.dll.force_stub || []).join("\n");

  // Environment
  const envList = document.getElementById("env-list");
  envList.innerHTML = "";
  for (const [k, v] of Object.entries(config.environment || {})) {
    addKvRow(envList, k, v, "VARIABLE", "Name");
  }
}

function readForm() {
  if (!config) return;

  // Windows version
  const sel = document.getElementById("win-version");
  config.windows_version = JSON.parse(sel.value);

  // Filesystem
  const root = document.getElementById("default-root").value.trim();
  config.filesystem.default_root = root || null;
  config.filesystem.case_insensitive = document.getElementById("case-insensitive").checked;

  // Drives
  config.filesystem.drives = {};
  for (const row of document.getElementById("drive-list").querySelectorAll(".kv-row")) {
    const k = row.querySelector(".kv-key").value.trim().toUpperCase();
    const v = row.querySelector(".kv-value").value.trim();
    if (k && v) config.filesystem.drives[k] = v;
  }

  // DLL
  config.dll.search_order = textareaToList("search-order");
  config.dll.force_stub = textareaToList("force-stub");

  // Environment
  config.environment = {};
  for (const row of document.getElementById("env-list").querySelectorAll(".kv-row")) {
    const k = row.querySelector(".kv-key").value.trim();
    const v = row.querySelector(".kv-value").value.trim();
    if (k) config.environment[k] = v;
  }
}

async function saveConfig() {
  readForm();
  try {
    await invoke("save_config_cmd", { exePath, config });
    showStatus("Saved", false);
  } catch (err) {
    showStatus("Save failed: " + err, true);
  }
}

// ---------------------------------------------------------------------------
// UI Helpers
// ---------------------------------------------------------------------------

function setupTabs() {
  for (const btn of document.querySelectorAll(".tab")) {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".tab").forEach(b => b.classList.remove("active"));
      document.querySelectorAll(".tab-content").forEach(s => s.classList.remove("active"));
      btn.classList.add("active");
      document.getElementById("tab-" + btn.dataset.tab).classList.add("active");
    });
  }
}

function setupButtons() {
  document.getElementById("save-btn").addEventListener("click", saveConfig);

  document.getElementById("reset-btn").addEventListener("click", () => {
    config = {
      filesystem: { default_root: null, drives: {}, case_insensitive: false },
      windows_version: "win11",
      dll: { search_order: [], force_stub: [] },
      environment: {},
    };
    populateForm();
    showStatus("Reset to defaults (not saved yet)", false);
  });

  document.getElementById("add-drive").addEventListener("click", () => {
    addKvRow(document.getElementById("drive-list"), "", "", "A-Z", "Drive letter");
  });

  document.getElementById("add-env").addEventListener("click", () => {
    addKvRow(document.getElementById("env-list"), "", "", "VARIABLE", "Name");
  });

  document.getElementById("launch-btn").addEventListener("click", async () => {
    if (!exePath) return;
    const box = document.getElementById("runner-output");
    box.textContent = "Launching…";
    try {
      const output = await invoke("launch_exe", { exePath });
      box.textContent = output;
    } catch (err) {
      box.textContent = "Error: " + err;
    }
  });
}

function addKvRow(container, key, value, keyPlaceholder, keyLabel) {
  const row = document.createElement("div");
  row.className = "kv-row";
  row.innerHTML = `
    <input type="text" class="kv-key" value="${escHtml(key)}" placeholder="${escHtml(keyPlaceholder)}" aria-label="${escHtml(keyLabel)}">
    <input type="text" class="kv-value" value="${escHtml(value)}" placeholder="Value">
    <button class="btn-remove" title="Remove">&times;</button>
  `;
  row.querySelector(".btn-remove").addEventListener("click", () => row.remove());
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

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------

document.addEventListener("DOMContentLoaded", init);
