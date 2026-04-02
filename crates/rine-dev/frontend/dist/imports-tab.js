// Imports tab logic.

function renderImportTable(imp) {
  let rows = [];
  for (const dll of imp.summaries) {
    for (const name of dll.resolved_names) {
      rows.push({ dll: dll.dll_name, name, stub: false });
    }
    for (const name of dll.stubbed_names) {
      rows.push({ dll: dll.dll_name, name, stub: true });
    }
  }

  rows.sort((a, b) => {
    if (a.stub !== b.stub) return a.stub ? -1 : 1;
    return a.dll.localeCompare(b.dll) || a.name.localeCompare(b.name);
  });

  window._importRows = rows;
  applyImportFilter();
}

function applyImportFilter() {
  const rows = window._importRows || [];
  const filterText = (document.getElementById('import-filter').value || '').toLowerCase();
  const stubsOnly = document.getElementById('import-stubs-only').checked;
  const tbody = document.getElementById('import-tbody');

  const filtered = rows.filter(r => {
    if (stubsOnly && !r.stub) return false;
    if (filterText) {
      return r.dll.toLowerCase().includes(filterText) || r.name.toLowerCase().includes(filterText);
    }
    return true;
  });

  tbody.innerHTML = filtered.map(r =>
    `<tr>
      <td>${esc(r.dll)}</td>
      <td>${esc(r.name)}</td>
      <td class="${r.stub ? 'status-stub' : 'status-resolved'}">${r.stub ? 'Stub' : 'OK'}</td>
    </tr>`
  ).join('') || '<tr><td colspan="3" class="placeholder">No matching imports</td></tr>';
}

function bindImportsUi() {
  document.getElementById('import-filter').addEventListener('input', applyImportFilter);
  document.getElementById('import-stubs-only').addEventListener('change', applyImportFilter);
}
