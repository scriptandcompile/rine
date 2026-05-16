// rine-config frontend — Tauri IPC via window.__TAURI__
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let exePath = null;
let configPath = null;
let config = null;
let dirty = false;
let editorObserversBound = false;

function markDirty() {
  if (dirty) return;
  dirty = true;
  invoke("set_menu_enabled", { id: "save", enabled: true });
  invoke("set_menu_enabled", { id: "reset", enabled: true });
}

function markClean() {
  dirty = false;
  invoke("set_menu_enabled", { id: "save", enabled: false });
  invoke("set_menu_enabled", { id: "reset", enabled: false });
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

async function init() {
  // Ask the backend for the initial open path passed as a CLI argument.
  let openPath = null;
  try {
    openPath = await invoke("get_exe_path");
  } catch (_) {
    // ignore — will show the welcome screen
  }

  if (openPath) {
    await openConfigTarget(openPath);
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
  return openConfigTarget(path);
}

async function openConfigTarget(path) {
  try {
    const opened = await invoke("open_config_target", { path });
    config = opened.config;
    exePath = opened.exe_path;
    configPath = opened.config_path;

    document.getElementById("no-exe").classList.add("hidden");
    document.getElementById("editor").classList.remove("hidden");

    document.getElementById("exe-path-display").textContent = exePath || "(direct config file)";
    document.getElementById("config-path-display").textContent = configPath;
    document.getElementById("launch-btn").disabled = !exePath;
    setRunnerTabEnabled(Boolean(exePath));

    await populateVersions();
    populateForm();
    markClean();
    observeChanges();
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
    if (exePath) {
      await invoke("save_config_cmd", { exePath, config });
    } else if (configPath) {
      await invoke("save_config_file", { configPath, config });
    } else {
      throw new Error("No configuration target selected");
    }
    markClean();
    showStatus("Saved", false);
  } catch (err) {
    showStatus("Save failed: " + err, true);
  }
}

function observeChanges() {
  if (editorObserversBound) return;
  editorObserversBound = true;
  const editor = document.getElementById("editor");
  editor.addEventListener("input", markDirty);
  editor.addEventListener("change", markDirty);
}

// ---------------------------------------------------------------------------
// UI Helpers
// ---------------------------------------------------------------------------

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

function setupButtons() {
  // Listen for File > Save from native menu
  listen("menu-save", () => saveConfig());

  // Listen for exe dropped onto window (from drag-and-drop or CLI startup via backend emit)
  listen("exe-path-dropped", async (event) => {
    const droppedPath = event && typeof event.payload === "string" ? event.payload : null;
    if (!droppedPath) return;
    await loadConfig(droppedPath);
    showStatus("Loaded dropped executable", false);
  });

  listen("open-path-selected", async (event) => {
    const selectedPath = event && typeof event.payload === "string" ? event.payload : null;
    if (!selectedPath) return;
    await openConfigTarget(selectedPath);
    showStatus("Loaded selected target", false);
  });

  // Listen for File > Reset to Defaults from native menu
  listen("menu-reset", () => {
    config = {
      filesystem: { default_root: null, drives: {}, case_insensitive: true },
      windows_version: "win11",
      dll: { search_order: [], force_stub: [] },
      dialogs: {
        theme: "native",
        native_backend: "auto",
      },
      environment: {},
    };
    populateForm();
    markDirty();
    showStatus("Reset to defaults (not saved yet)", false);
  });

  document.getElementById("add-drive").addEventListener("click", () => {
    addKvRow(document.getElementById("drive-list"), "", "", "A-Z", "Drive letter");
    markDirty();
  });

  document.getElementById("add-env").addEventListener("click", () => {
    addKvRow(document.getElementById("env-list"), "", "", "VARIABLE", "Name");
    markDirty();
  });

  document.getElementById("browse-root").addEventListener("click", async () => {
    const input = document.getElementById("default-root");
    const startDir = input.value.trim() || "~/.rine/drives";
    try {
      const folder = await invoke("pick_folder", { startDir });
      if (folder) {
        input.value = folder;
        markDirty();
      }
    } catch (err) {
      // user cancelled or error — ignore
    }
  });

  document.getElementById("launch-btn").addEventListener("click", async () => {
    if (!exePath) return;
    const stdoutBox = document.getElementById("runner-stdout");
    const stderrBox = document.getElementById("runner-stderr");
    const exitDisplay = document.getElementById("exit-code-display");
    stdoutBox.innerHTML = "Launching…";
    stderrBox.innerHTML = "";
    exitDisplay.classList.add("hidden");
    try {
      const { stdout, stderr, exit_code } = await invoke("launch_exe", { exePath });
      stdoutBox.innerHTML = stdout ? ansiToHtml(stdout) : "<em>no output</em>";
      stderrBox.innerHTML = stderr ? ansiToHtml(stderr) : "<em>no output</em>";
      exitDisplay.textContent = "exit " + exit_code;
      exitDisplay.className = "exit-code" + (exit_code !== 0 ? " error" : "");
    } catch (err) {
      stdoutBox.textContent = "Error: " + err;
      stderrBox.innerHTML = "";
    }
  });

  // Runner sub-tabs
  for (const btn of document.querySelectorAll(".runner-tab")) {
    btn.addEventListener("click", () => {
      document.querySelectorAll(".runner-tab").forEach(b => b.classList.remove("active"));
      document.querySelectorAll(".runner-pane").forEach(p => p.classList.remove("active"));
      btn.classList.add("active");
      document.getElementById("runner-" + btn.dataset.output).classList.add("active");
    });
  }

  // Registry tab
  const registryReloadBtn = document.getElementById("registry-reload");
  if (registryReloadBtn) {
    registryReloadBtn.addEventListener("click", loadRegistryData);
  }

  // Load registry data when registry tab is shown
  const registryTab = document.querySelector('.tab[data-tab="registry"]');
  if (registryTab) {
    const originalClickHandler = registryTab.onclick;
    registryTab.addEventListener("click", async () => {
      if (!registryData && exePath) {
        await loadRegistryData();
      } else if (registryData) {
        renderRegistry();
      }
    });
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

// ---------------------------------------------------------------------------
// Registry Viewer/Editor
// ---------------------------------------------------------------------------

let registryData = null;
let registryTree = {};

async function loadRegistryData() {
  if (!exePath) {
    showStatus("No executable selected", true);
    return;
  }

  try {
    const data = await invoke("get_registry_export", { exePath });
    registryData = data;
    renderRegistry();
    showStatus("Registry loaded", false);
  } catch (err) {
    showStatus("Failed to load registry: " + err, true);
  }
}

function renderRegistry() {
  if (!registryData || !registryData.roots) return;

  const treeContainer = document.getElementById("registry-tree");
  treeContainer.innerHTML = "";

  const rootEntries = Object.entries(registryData.roots);
  if (rootEntries.length === 0) {
    treeContainer.innerHTML = "<div style='padding: 12px; color: var(--text-muted);'>No registry data</div>";
    return;
  }

  const lockedPaths = registryData.locked_paths || [];
  for (const [, rootKey] of rootEntries) {
    renderKeyNode(rootKey, treeContainer, lockedPaths);
  }
}

function renderKeyNode(keyData, container, lockedPaths) {
  const nodeEl = document.createElement("div");
  nodeEl.className = "registry-node";
  nodeEl.dataset.path = keyData.path;

  // Render the key itself (with expand/collapse if it has children)
  if (keyData.subkey_names && keyData.subkey_names.length > 0) {
    const keyBtn = document.createElement("div");
    keyBtn.className = "registry-key collapsed";
    keyBtn.innerHTML = `<span class="registry-key-name">${escHtml(keyData.path)}</span>`;
    
    const childrenDiv = document.createElement("div");
    childrenDiv.className = "registry-key-children";
    
    // Lazy load subkeys on expand
    let loaded = false;
    keyBtn.addEventListener("click", async () => {
      if (!loaded && childrenDiv.innerHTML === "") {
        // Load subkeys
        await loadAndRenderSubkeys(keyData, childrenDiv, lockedPaths);
        loaded = true;
      }
      keyBtn.classList.toggle("expanded");
      keyBtn.classList.toggle("collapsed");
    });
    
    nodeEl.appendChild(keyBtn);
    nodeEl.appendChild(childrenDiv);
  } else {
    const keyDiv = document.createElement("div");
    keyDiv.className = "registry-key collapsed";
    keyDiv.innerHTML = `<span class="registry-key-name">${escHtml(keyData.path)}</span>`;
    nodeEl.appendChild(keyDiv);
  }

  // Render values
  if (keyData.values && keyData.values.length > 0) {
    for (const value of keyData.values) {
      renderValue(value, nodeEl, lockedPaths);
    }
  }

  container.appendChild(nodeEl);
}

function renderValue(valueData, container, lockedPaths) {
  const valueEl = document.createElement("div");
  const valueName = valueData.name || "(Default)";
  const valuePath = container.dataset.path + "\\" + valueName;
  const isLocked = valueData.locked || lockedPaths.some(p => 
    p.toLowerCase() === valuePath.toLowerCase()
  );

  valueEl.className = "registry-value" + (isLocked ? " locked" : "");
  
  const nameSpan = document.createElement("span");
  nameSpan.className = "registry-value-name" + (valueData.name === "" ? " default" : "");
  nameSpan.textContent = valueData.name === "" ? "(Default)" : valueData.name;

  const typeSpan = document.createElement("span");
  typeSpan.className = "registry-value-type";
  typeSpan.textContent = valueData.type_name;

  const dataSpan = document.createElement("span");
  dataSpan.className = "registry-value-data";

  // Keep all values visible; locked values are rendered as read-only inputs.
  const input = document.createElement("input");
  input.type = "text";
  input.value = valueData.data;

  if (isLocked) {
    input.disabled = true;
    input.readOnly = true;
    input.title = "Locked to Windows version";
    dataSpan.appendChild(input);

    const badge = document.createElement("span");
    badge.className = "registry-lock-badge";
    badge.textContent = "LOCKED";
    dataSpan.appendChild(badge);
  } else {
    input.addEventListener("change", () => {
      markDirty();
      // TODO: Save to registry
    });
    dataSpan.appendChild(input);
  }

  valueEl.appendChild(nameSpan);
  valueEl.appendChild(typeSpan);
  valueEl.appendChild(dataSpan);
  container.appendChild(valueEl);
}

async function loadAndRenderSubkeys(keyData, container, lockedPaths) {
  // TODO: Load and render subkeys
  container.innerHTML = "<div style='padding: 8px; color: var(--text-muted);'>(Subkey loading not yet implemented)</div>";
}

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------

document.addEventListener("DOMContentLoaded", init);
