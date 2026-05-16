// Registry tab: view and edit Windows registry data

let registryData = null;
let registryTree = {};

function setupRegistryTab() {
  const registryReloadBtn = document.getElementById("registry-reload");
  if (registryReloadBtn) {
    registryReloadBtn.addEventListener("click", loadRegistryData);
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
  if (!exePath) {
    showStatus("No executable selected", true);
    return;
  }

  try {
    const data = await window.__TAURI__.core.invoke("get_registry_export", { exePath });
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
