const supportBody = document.getElementById("support-body");
const dllFilter = document.getElementById("dll-filter");
const x64Filter = document.getElementById("x64-filter");
const x86Filter = document.getElementById("x86-filter");
const searchInput = document.getElementById("search-input");

const dllCount = document.getElementById("dll-count");
const functionCount = document.getElementById("function-count");
const x64Implemented = document.getElementById("x64-implemented");
const x86Implemented = document.getElementById("x86-implemented");
const x64TotalImplemented = document.getElementById("x64-total-implemented");
const x64TotalPartial = document.getElementById("x64-total-partial");
const x64TotalStubbed = document.getElementById("x64-total-stubbed");
const x64TotalUnimplemented = document.getElementById("x64-total-unimplemented");
const x86TotalImplemented = document.getElementById("x86-total-implemented");
const x86TotalPartial = document.getElementById("x86-total-partial");
const x86TotalStubbed = document.getElementById("x86-total-stubbed");
const x86TotalUnimplemented = document.getElementById("x86-total-unimplemented");
const generatedAt = document.getElementById("generated-at");

const GITHUB_BLOB_ROOT = "https://github.com/scriptandcompile/rine/blob/main";
const SOURCE_ROOTS = {
  x64: "crates/platform/win64-dll",
  x86: "crates/platform/win32-dll",
};

let rows = [];
const STATUS_ORDER = ["implemented", "partial", "stubbed", "unimplemented"];

function normalizeStatus(value) {
  const normalized = String(value || "").trim().toLowerCase();
  if (STATUS_ORDER.includes(normalized)) {
    return normalized;
  }

  if (normalized === "partially implemented" || normalized.includes("partial")) {
    return "partial";
  }

  if (normalized === "stub" || normalized === "stubs" || normalized.includes("stubbed")) {
    return "stubbed";
  }

  if (normalized.includes("unimplemented")) {
    return "unimplemented";
  }

  return "implemented";
}

function safeStatusTotal(statusTotals, status) {
  if (!statusTotals || typeof statusTotals !== "object") {
    return 0;
  }
  return Number(statusTotals[status] || 0);
}

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
  const tooltips = {
    implemented: "Fully functional and feature complete",
    partial: "Partially implemented with some missing features",
    stubbed: "Returns default values, allows programs to continue",
    unimplemented: "Not implemented",
  };
  const tooltip = tooltips[status] || status;
  return `<span class="status-pill ${status}" data-tooltip="${escapeHtml(tooltip)}">${status}</span>`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

function extractSourceFileName(sourceLabel) {
  if (!sourceLabel || typeof sourceLabel !== "string") {
    return null;
  }

  const parts = sourceLabel.split(" - ");
  if (parts.length >= 2) {
    return parts.slice(1).join(" - ").trim();
  }

  return sourceLabel.trim();
}

function buildSourceUrl(arch, dllName, fileName) {
  const sourceRoot = SOURCE_ROOTS[arch];
  if (!sourceRoot || !dllName || !fileName) {
    return null;
  }

  const dllBase = dllName.toLowerCase().replace(/\.dll$/, "");
  const cratePrefix = arch === "x64" ? "rine64-" : "rine32-";
  const repoPath = `${sourceRoot}/${cratePrefix}${dllBase}/src/${fileName}`;
  return `${GITHUB_BLOB_ROOT}/${repoPath}`;
}

function createSourceCell(arch, dllName, archData) {
  if (!archData || !archData.source) {
    return "<span class=\"source-path\">-</span>";
  }

  const fileName = extractSourceFileName(archData.source);
  if (!fileName) {
    return "<span class=\"source-path\">-</span>";
  }

  const sourceUrl = buildSourceUrl(arch, dllName, fileName);
  const safeFileName = escapeHtml(fileName);

  if (!sourceUrl) {
    return `<span class="source-path">${safeFileName}</span>`;
  }

  return `<a class="source-path source-link" href="${sourceUrl}" target="_blank" rel="noopener noreferrer">${safeFileName}</a>`;
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

  let previousDll = "";
  const markup = filtered
    .map((row) => {
      const groupDivider =
        row.dll !== previousDll
          ? `<tr class="dll-divider-row"><th colspan="6" scope="colgroup">${escapeHtml(row.dll)}</th></tr>`
          : "";

      previousDll = row.dll;

      return `${groupDivider}
      <tr>
        <td><span class="dll-name">${row.dll}</span></td>
        <td>${row.name}</td>
        <td>${createStatusPill(row.x64.status)}</td>
        <td>${createStatusPill(row.x86.status)}</td>
        <td>${createSourceCell("x64", row.dll, row.x64)}</td>
        <td>${createSourceCell("x86", row.dll, row.x86)}</td>
      </tr>`;
    })
    .join("");

  supportBody.innerHTML = markup;
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
    x64Implemented.textContent = String(safeStatusTotal(data.totals.x64, "implemented"));
    x86Implemented.textContent = String(safeStatusTotal(data.totals.x86, "implemented"));
    x64TotalImplemented.textContent = String(safeStatusTotal(data.totals.x64, "implemented"));
    x64TotalPartial.textContent = String(safeStatusTotal(data.totals.x64, "partial"));
    x64TotalStubbed.textContent = String(safeStatusTotal(data.totals.x64, "stubbed"));
    x64TotalUnimplemented.textContent = String(safeStatusTotal(data.totals.x64, "unimplemented"));
    x86TotalImplemented.textContent = String(safeStatusTotal(data.totals.x86, "implemented"));
    x86TotalPartial.textContent = String(safeStatusTotal(data.totals.x86, "partial"));
    x86TotalStubbed.textContent = String(safeStatusTotal(data.totals.x86, "stubbed"));
    x86TotalUnimplemented.textContent = String(safeStatusTotal(data.totals.x86, "unimplemented"));
    generatedAt.textContent = `Generated: ${formatGeneratedAt(data.generatedAt)}`;

    const dllNames = data.dlls.map((dll) => dll.name).sort((a, b) => a.localeCompare(b));
    dllNames.forEach((name) => {
      const option = document.createElement("option");
      option.value = name;
      option.textContent = name;
      dllFilter.append(option);
    });

    rows = data.dlls
      .flatMap((dll) =>
        dll.functions.map((fn) => ({
          dll: dll.name,
          ...fn,
          x64: {
            ...(fn.x64 || {}),
            status: normalizeStatus(fn.x64?.status),
          },
          x86: {
            ...(fn.x86 || {}),
            status: normalizeStatus(fn.x86?.status),
          },
        }))
      )
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
