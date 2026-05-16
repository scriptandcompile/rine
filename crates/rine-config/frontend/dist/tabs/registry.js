// Registry tab: view and edit Windows registry data

let registryData = null;
let registryTree = {};
const REGISTRY_SAVE_DEBOUNCE_MS = 500;
const pendingRegistrySaves = new Map();
const pendingRegistryTimers = new Map();
const activeRegistrySavePromises = new Set();
const LOCKED_VALUE_TOOLTIP = "This value is locked to the selected Windows version default registry data. To use a different default Windows registry profile, change General -> Windows Version.";

function setupRegistryTab() {
  const winVersionSel = document.getElementById("win-version");
  if (winVersionSel) {
    winVersionSel.addEventListener("change", async () => {
      clearPendingRegistrySaves();
      registryData = null;
      registryTree = {};

      const registryPanel = document.getElementById("tab-registry");
      if (exePath && registryPanel && registryPanel.classList.contains("active")) {
        await loadRegistryData();
      }
    });
  }

  // Load registry data when registry tab is shown
  const registryTab = document.querySelector('.tab[data-tab="registry"]');
  if (registryTab) {
    registryTab.addEventListener("click", async () => {
      if (!registryData && exePath) {
        await loadRegistryData();
      } else if (registryData) {
        renderRegistry();
      }
    });
  }
}

async function loadRegistryData() {
  clearPendingRegistrySaves();

  if (!exePath) {
    showStatus("No executable selected", true);
    return;
  }

  try {
    const data = await window.__TAURI__.core.invoke("get_registry_export", {
      exePath,
      windowsVersion: getSelectedWindowsVersion(),
    });
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
  const keyLabel = getRegistryKeyLabel(keyData.path);
  const hasValues = keyData.values && keyData.values.length > 0;

  // Render the key itself (with expand/collapse if it has children)
  if (keyData.subkey_names && keyData.subkey_names.length > 0) {
    const keyBtn = document.createElement("div");
    keyBtn.className = "registry-key collapsed";
    keyBtn.innerHTML = `<span class="registry-key-name">${escHtml(keyLabel)}</span>`;
    
    const childrenDiv = document.createElement("div");
    childrenDiv.className = "registry-key-children";
    
    // Lazy load subkeys on expand
    let loaded = false;
    const ensureExpanded = async () => {
      if (!loaded && childrenDiv.innerHTML === "") {
        // Load subkeys
        await loadAndRenderSubkeys(keyData, childrenDiv, lockedPaths);
        loaded = true;
      }
      if (!keyBtn.classList.contains("expanded")) {
        keyBtn.classList.add("expanded");
        keyBtn.classList.remove("collapsed");
      }
    };

    keyBtn.__ensureExpanded = ensureExpanded;

    keyBtn.addEventListener("click", async () => {
      if (keyBtn.classList.contains("expanded")) {
        keyBtn.classList.remove("expanded");
        keyBtn.classList.add("collapsed");
        return;
      }

      await ensureExpanded();
      await autoExpandSingleChildChain(childrenDiv);
    });
    
    nodeEl.appendChild(keyBtn);
    nodeEl.appendChild(childrenDiv);
  } else {
    const keyDiv = document.createElement("div");
    keyDiv.className = "registry-key " + (hasValues ? "expanded" : "collapsed");
    keyDiv.innerHTML = `<span class="registry-key-name">${escHtml(keyLabel)}</span>`;
    nodeEl.appendChild(keyDiv);
  }

  // Render values
  if (hasValues) {
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
    input.title = LOCKED_VALUE_TOOLTIP;
    valueEl.title = LOCKED_VALUE_TOOLTIP;
    dataSpan.appendChild(input);

    const badge = document.createElement("span");
    badge.className = "registry-lock-badge";
    badge.textContent = "LOCKED";
    badge.title = LOCKED_VALUE_TOOLTIP;
    dataSpan.appendChild(badge);
  } else {
    const keyPath = container.dataset.path;
    const onValueEdited = (immediate) => {
      scheduleRegistryValueSave({
        keyPath,
        valueName: valueData.name,
        input,
      }, immediate);
    };
    input.addEventListener("input", () => onValueEdited(false));
    input.addEventListener("change", () => onValueEdited(true));
    dataSpan.appendChild(input);
  }

  valueEl.appendChild(nameSpan);
  valueEl.appendChild(typeSpan);
  valueEl.appendChild(dataSpan);
  container.appendChild(valueEl);
}

function registrySaveId(keyPath, valueName) {
  return `${keyPath}\\${valueName}`;
}

function scheduleRegistryValueSave(saveRequest, immediate) {
  const saveId = registrySaveId(saveRequest.keyPath, saveRequest.valueName);
  pendingRegistrySaves.set(saveId, saveRequest);

  const existingTimer = pendingRegistryTimers.get(saveId);
  if (existingTimer) {
    clearTimeout(existingTimer);
  }

  if (immediate) {
    pendingRegistryTimers.delete(saveId);
    void startRegistryValueSave(saveId);
    return;
  }

  const timerId = setTimeout(() => {
    pendingRegistryTimers.delete(saveId);
    void startRegistryValueSave(saveId);
  }, REGISTRY_SAVE_DEBOUNCE_MS);
  pendingRegistryTimers.set(saveId, timerId);
}

function startRegistryValueSave(saveId) {
  const promise = persistRegistryValue(saveId);
  activeRegistrySavePromises.add(promise);
  promise.finally(() => {
    activeRegistrySavePromises.delete(promise);
  });
  return promise;
}

async function persistRegistryValue(saveId) {
  const saveRequest = pendingRegistrySaves.get(saveId);
  if (!saveRequest) {
    return true;
  }

  const latestValue = saveRequest.input.value;
  try {
    await window.__TAURI__.core.invoke("update_registry_value", {
      exePath,
      keyPath: saveRequest.keyPath,
      valueName: saveRequest.valueName,
      newValue: latestValue,
      windowsVersion: getSelectedWindowsVersion(),
    });

    const currentRequest = pendingRegistrySaves.get(saveId);
    if (currentRequest === saveRequest && currentRequest.input.value === latestValue) {
      pendingRegistrySaves.delete(saveId);
    }
    return true;
  } catch (err) {
    showStatus("Failed to save registry value: " + err, true);
    return false;
  }
}

function clearPendingRegistrySaves() {
  for (const timerId of pendingRegistryTimers.values()) {
    clearTimeout(timerId);
  }
  pendingRegistryTimers.clear();
  pendingRegistrySaves.clear();
}

async function flushPendingRegistrySaves() {
  for (const timerId of pendingRegistryTimers.values()) {
    clearTimeout(timerId);
  }
  pendingRegistryTimers.clear();

  const saveIds = Array.from(pendingRegistrySaves.keys());
  const saveResults = await Promise.all(saveIds.map((saveId) => startRegistryValueSave(saveId)));
  const inFlightResults = await Promise.all(Array.from(activeRegistrySavePromises));
  return saveResults.every(Boolean) && inFlightResults.every(Boolean);
}

function getRegistryKeyLabel(fullPath) {
  if (!fullPath) return "";
  const parts = String(fullPath).split("\\").filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : String(fullPath);
}

function getSelectedWindowsVersion() {
  const sel = document.getElementById("win-version");
  if (!sel || !sel.value) return null;

  try {
    return JSON.parse(sel.value);
  } catch (_err) {
    return null;
  }
}

function getDirectChildrenByClass(parent, className) {
  return Array.from(parent.children).filter((child) => child.classList.contains(className));
}

function getDirectChildByClass(parent, className) {
  return Array.from(parent.children).find((child) => child.classList.contains(className));
}

async function autoExpandSingleChildChain(startContainer) {
  let container = startContainer;

  while (container) {
    const directNodes = getDirectChildrenByClass(container, "registry-node");
    if (directNodes.length !== 1) {
      return;
    }

    const onlyNode = directNodes[0];
    const childKeyBtn = getDirectChildByClass(onlyNode, "registry-key");
    const childChildrenDiv = getDirectChildByClass(onlyNode, "registry-key-children");

    // Leaf nodes have no expandable children, so stop walking.
    if (!childKeyBtn || !childChildrenDiv) {
      return;
    }

    const ensureExpanded = childKeyBtn.__ensureExpanded;
    if (typeof ensureExpanded !== "function") {
      return;
    }

    await ensureExpanded();
    container = childChildrenDiv;
  }
}

async function loadAndRenderSubkeys(keyData, container, lockedPaths) {
  if (!exePath) {
    container.innerHTML = "<div style='padding: 8px; color: var(--danger);'>No executable selected</div>";
    return;
  }

  try {
    const currentKey = await window.__TAURI__.core.invoke("get_registry_key", {
      exePath,
      keyPath: keyData.path,
      windowsVersion: getSelectedWindowsVersion(),
    });

    const subkeyNames = currentKey.subkey_names || [];
    if (subkeyNames.length === 0) {
      container.innerHTML = "<div style='padding: 8px; color: var(--text-muted);'>(No subkeys)</div>";
      return;
    }

    const childNodes = await Promise.all(
      subkeyNames.map(async (subkeyName) => {
        const childPath = `${keyData.path}\\${subkeyName}`;
        try {
          return await window.__TAURI__.core.invoke("get_registry_key", {
            exePath,
            keyPath: childPath,
            windowsVersion: getSelectedWindowsVersion(),
          });
        } catch (_err) {
          return {
            path: childPath,
            values: [],
            subkey_names: [],
          };
        }
      })
    );

    container.innerHTML = "";
    for (const childKey of childNodes) {
      renderKeyNode(childKey, container, lockedPaths);
    }
  } catch (err) {
    container.innerHTML = `<div style='padding: 8px; color: var(--danger);'>Failed to load subkeys: ${escHtml(String(err))}</div>`;
  }
}
