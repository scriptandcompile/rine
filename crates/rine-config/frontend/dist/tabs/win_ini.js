// WIN.INI tab: load and edit global/per-app WIN.INI files

let winIniCacheKey = null;
const WIN_INI_SAVE_DEBOUNCE_MS = 500;
const pendingWinIniSaves = new Map();
const pendingWinIniTimers = new Map();
const activeWinIniSavePromises = new Set();

function clearWinIniViewState() {
  clearPendingWinIniSaves();
  winIniCacheKey = null;

  const globalWinIniEl = document.getElementById("global-win-ini");
  if (globalWinIniEl) {
    globalWinIniEl.value = "";
    globalWinIniEl.dataset.targetPath = "";
  }

  const appWinIniEl = document.getElementById("app-win-ini");
  if (appWinIniEl) {
    appWinIniEl.value = "";
    appWinIniEl.dataset.targetPath = "";
  }
}

function setupWinIniTab() {
  setupWinIniEditors();

  const winIniTab = document.querySelector('.tab[data-tab="winini"]');
  if (winIniTab) {
    winIniTab.addEventListener("click", async () => {
      await ensureWinIniLoaded();
    });
  }

}

function getWinIniCacheKey() {
  return exePath || "";
}

function setupWinIniEditors() {
  setupWinIniEditor("global-win-ini", "global");
  setupWinIniEditor("app-win-ini", "app");
}

function setupWinIniEditor(textAreaId, scope) {
  const textArea = document.getElementById(textAreaId);
  if (!textArea) return;

  const onWinIniEdited = (immediate) => {
    if (textArea.disabled) {
      return;
    }

    scheduleWinIniSave({
      scope,
      textArea,
      content: textArea.value,
    }, immediate);
  };

  textArea.addEventListener("input", () => onWinIniEdited(false));
  textArea.addEventListener("change", () => onWinIniEdited(true));
  textArea.addEventListener("blur", () => onWinIniEdited(true));

}

function scheduleWinIniSave(saveRequest, immediate) {
  pendingWinIniSaves.set(saveRequest.scope, saveRequest);

  const existingTimer = pendingWinIniTimers.get(saveRequest.scope);
  if (existingTimer) {
    clearTimeout(existingTimer);
  }

  if (immediate) {
    pendingWinIniTimers.delete(saveRequest.scope);
    void startWinIniSave(saveRequest.scope);
    return;
  }

  const timerId = setTimeout(() => {
    pendingWinIniTimers.delete(saveRequest.scope);
    void startWinIniSave(saveRequest.scope);
  }, WIN_INI_SAVE_DEBOUNCE_MS);
  pendingWinIniTimers.set(saveRequest.scope, timerId);
}

function startWinIniSave(scope) {
  const promise = persistWinIniContent(scope);
  activeWinIniSavePromises.add(promise);
  promise.finally(() => {
    activeWinIniSavePromises.delete(promise);
  });
  return promise;
}

async function persistWinIniContent(scope) {
  const saveRequest = pendingWinIniSaves.get(scope);
  if (!saveRequest) {
    return true;
  }

  try {
    const savedPath = await saveWinIniContent(scope, saveRequest.content);
    const currentRequest = pendingWinIniSaves.get(scope);
    if (currentRequest === saveRequest) {
      pendingWinIniSaves.delete(scope);
      if (saveRequest.textArea) {
        saveRequest.textArea.dataset.targetPath = savedPath || "";
      }
      updateWinIniPathLabel(scope, savedPath || "");
    }
    return true;
  } catch (err) {
    showStatus("Failed to save " + scope.toUpperCase() + " WIN.INI: " + err, true);
    return false;
  }
}

function clearPendingWinIniSaves() {
  for (const timerId of pendingWinIniTimers.values()) {
    clearTimeout(timerId);
  }
  pendingWinIniTimers.clear();
  pendingWinIniSaves.clear();
}

async function flushPendingWinIniSaves() {
  for (const timerId of pendingWinIniTimers.values()) {
    clearTimeout(timerId);
  }
  pendingWinIniTimers.clear();

  const winIniScopes = Array.from(pendingWinIniSaves.keys());
  const winIniResults = await Promise.all(winIniScopes.map((scope) => startWinIniSave(scope)));
  const winIniInFlightResults = await Promise.all(Array.from(activeWinIniSavePromises));

  return winIniResults.every(Boolean) && winIniInFlightResults.every(Boolean);
}

async function ensureWinIniLoaded() {
  const cacheKey = getWinIniCacheKey();
  if (cacheKey === winIniCacheKey) {
    return;
  }

  await Promise.all([
    loadWinIniEditor("global", "global-win-ini"),
    loadWinIniEditor("app", "app-win-ini"),
  ]);

  winIniCacheKey = cacheKey;
}

async function loadWinIniEditor(scope, textAreaId) {
  const textArea = document.getElementById(textAreaId);
  if (!textArea) return;

  if (scope === "app" && !exePath) {
    textArea.value = "";
    textArea.placeholder = "Per-app WIN.INI requires an executable target.";
    textArea.disabled = true;
    textArea.dataset.targetPath = "";
    updateWinIniPathLabel(scope, "No executable target selected");
    return;
  }

  try {
    const result = await window.__TAURI__.core.invoke("load_win_ini_text", {
      exePath,
      scope,
    });

    textArea.disabled = false;
    textArea.value = result.content || "";
    textArea.dataset.targetPath = result.path || "";
    updateWinIniPathLabel(scope, result.path || "");
  } catch (err) {
    textArea.value = "";
    textArea.dataset.targetPath = "";
    updateWinIniPathLabel(scope, "Failed to load WIN.INI path");
    showStatus("Failed to load " + scope.toUpperCase() + " WIN.INI: " + err, true);
  }
}

async function saveWinIniContent(scope, content) {
  return await window.__TAURI__.core.invoke("save_win_ini_text", {
    exePath,
    scope,
    content,
  });
}

function updateWinIniPathLabel(scope, pathText) {
  const labelId = scope === "global" ? "global-win-ini-path" : "app-win-ini-path";
  const label = document.getElementById(labelId);
  if (!label) return;

  label.textContent = pathText || "";
}
