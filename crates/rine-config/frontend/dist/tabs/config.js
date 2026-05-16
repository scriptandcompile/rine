// Configuration loading, saving, and state management

const AUTOSAVE_DEBOUNCE_MS = 500;
let autosaveTimer = null;
let autosaveInFlight = false;
let autosaveQueued = false;

async function loadConfig(path) {
  return openConfigTarget(path);
}

async function openConfigTarget(path) {
  try {
    resetAutosaveState();

    const opened = await window.__TAURI__.core.invoke("open_config_target", { path });
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
  const versions = await window.__TAURI__.core.invoke("get_windows_versions");
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
      await window.__TAURI__.core.invoke("save_config_cmd", { exePath, config });
    } else if (configPath) {
      await window.__TAURI__.core.invoke("save_config_file", { configPath, config });
    } else {
      throw new Error("No configuration target selected");
    }
    markClean();
    return true;
  } catch (err) {
    showStatus("Save failed: " + err, true);
    return false;
  }
}

function scheduleAutosave() {
  if (!config) return;
  if (autosaveTimer) {
    clearTimeout(autosaveTimer);
  }
  autosaveTimer = setTimeout(() => {
    autosaveTimer = null;
    void runAutosave();
  }, AUTOSAVE_DEBOUNCE_MS);
}

async function runAutosave() {
  if (autosaveInFlight) {
    autosaveQueued = true;
    return;
  }

  autosaveInFlight = true;
  try {
    await saveConfig();
  } finally {
    autosaveInFlight = false;
    if (autosaveQueued) {
      autosaveQueued = false;
      await runAutosave();
    }
  }
}

function resetAutosaveState() {
  if (autosaveTimer) {
    clearTimeout(autosaveTimer);
    autosaveTimer = null;
  }
  autosaveInFlight = false;
  autosaveQueued = false;
}

async function flushPendingAutosave() {
  if (autosaveTimer) {
    clearTimeout(autosaveTimer);
    autosaveTimer = null;
  }

  if (dirty) {
    await runAutosave();
  }

  while (autosaveInFlight) {
    await new Promise((resolve) => setTimeout(resolve, 25));
  }

  return !dirty;
}

function observeChanges() {
  if (editorObserversBound) return;
  editorObserversBound = true;
  const editor = document.getElementById("editor");
  editor.addEventListener("input", markDirty);
  editor.addEventListener("change", markDirty);
}
