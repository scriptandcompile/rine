// rine-config frontend — Tauri IPC via window.__TAURI__
const { listen } = window.__TAURI__.event;

// Global state
let exePath = null;
let configPath = null;
let config = null;
let dirty = false;
let editorObserversBound = false;

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

async function init() {
  // Ask the backend for the initial open path passed as a CLI argument.
  let openPath = null;
  try {
    openPath = await window.__TAURI__.core.invoke("get_exe_path");
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
  setupMenuListeners();
  setupGeneralTab();
  setupRunnerTab();
  setupRegistryTab();
}

function setupMenuListeners() {
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
}

// ---------------------------------------------------------------------------
// Boot
// ---------------------------------------------------------------------------

document.addEventListener("DOMContentLoaded", init);
