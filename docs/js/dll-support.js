const supportBody = document.getElementById("support-body");
const dllFilter = document.getElementById("dll-filter");
const x64Filter = document.getElementById("x64-filter");
const x86Filter = document.getElementById("x86-filter");
const searchInput = document.getElementById("search-input");

const dllCount = document.getElementById("dll-count");
const functionCount = document.getElementById("function-count");
const x64Implemented = document.getElementById("x64-implemented");
const x86Implemented = document.getElementById("x86-implemented");
const generatedAt = document.getElementById("generated-at");

let rows = [];

function formatGeneratedAt(value) {
  if (!value) {
    return "-";
  }

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString();
}

function createStatusPill(status) {
  return `<span class="status-pill ${status}">${status}</span>`;
}

function createSourceCell(archData) {
  if (!archData || !archData.source) {
    return "<span class=\"source-path\">-</span>";
  }

  return `<span class="source-path">${archData.source}</span>`;
}

function renderTable() {
  const searchValue = (searchInput.value || "").trim().toLowerCase();
  const selectedDll = dllFilter.value;
  const selectedX64 = x64Filter.value;
  const selectedX86 = x86Filter.value;

  const filtered = rows.filter((row) => {
    if (selectedDll !== "all" && row.dll !== selectedDll) {
      return false;
    }

    if (selectedX64 !== "all" && row.x64.status !== selectedX64) {
      return false;
    }

    if (selectedX86 !== "all" && row.x86.status !== selectedX86) {
      return false;
    }

    if (searchValue.length > 0) {
      const haystack = `${row.dll} ${row.name}`.toLowerCase();
      return haystack.includes(searchValue);
    }

    return true;
  });

  if (filtered.length === 0) {
    supportBody.innerHTML = '<tr><td colspan="6" class="empty-state">No rows match the current filters.</td></tr>';
    return;
  }

  supportBody.innerHTML = filtered
    .map(
      (row) => `
      <tr>
        <td><span class="dll-name">${row.dll}</span></td>
        <td>${row.name}</td>
        <td>${createStatusPill(row.x64.status)}</td>
        <td>${createStatusPill(row.x86.status)}</td>
        <td>${createSourceCell(row.x64)}</td>
        <td>${createSourceCell(row.x86)}</td>
      </tr>`
    )
    .join("");
}

function wireFilters() {
  [dllFilter, x64Filter, x86Filter, searchInput].forEach((node) => {
    node.addEventListener("input", renderTable);
    node.addEventListener("change", renderTable);
  });
}

async function initialize() {
  try {
    const response = await fetch("data/dll-support.json", { cache: "no-store" });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }

    const data = await response.json();

    dllCount.textContent = String(data.dlls.length);
    functionCount.textContent = String(data.totals.functions);
    x64Implemented.textContent = String(data.totals.x64.implemented);
    x86Implemented.textContent = String(data.totals.x86.implemented);
    generatedAt.textContent = `Generated: ${formatGeneratedAt(data.generatedAt)}`;

    const dllNames = data.dlls.map((dll) => dll.name).sort((a, b) => a.localeCompare(b));
    dllNames.forEach((name) => {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      dllFilter.append(option);
    });

    rows = data.dlls
      .flatMap((dll) => dll.functions.map((fn) => ({ dll: dll.name, ...fn })))
      .sort((a, b) => {
        if (a.dll === b.dll) {
          return a.name.localeCompare(b.name);
        }
        return a.dll.localeCompare(b.dll);
      });

    wireFilters();
    renderTable();
  } catch (error) {
    supportBody.innerHTML = `<tr><td colspan="6" class="empty-state">Failed to load data: ${String(error)}</td></tr>`;
  }
}

initialize();
