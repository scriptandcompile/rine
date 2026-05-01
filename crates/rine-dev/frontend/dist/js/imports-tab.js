// Imports tab logic.

function importStatusLabel(kind) {
  switch (kind) {
    case 'implemented':
      return 'OK';
    case 'partial':
      return 'Partial';
    case 'stubbed':
      return 'Stubbed';
    case 'unimplemented':
      return 'Not Implemented';
    default:
      return kind;
  }
}

function importStatusClass(kind) {
  switch (kind) {
    case 'implemented':
      return 'status-resolved';
    case 'partial':
      return 'status-partial';
    case 'stubbed':
      return 'status-stubbed';
    case 'unimplemented':
      return 'status-unimplemented';
    default:
      return 'status-unknown';
  }
}

function renderImportTable(imp) {
  let rows = [];
  for (const dll of imp.summaries) {
    for (const entry of dll.imports) {
      rows.push({ dll: dll.dll_name, name: entry.name, kind: entry.kind });
    }
  }

  rows.sort((a, b) => {
    const order = {
      unimplemented: 0,
      stubbed: 1,
      partial: 2,
      implemented: 3,
    };
    if (a.kind !== b.kind) return (order[a.kind] ?? 99) - (order[b.kind] ?? 99);
    return a.dll.localeCompare(b.dll) || a.name.localeCompare(b.name);
  });

  window._importRows = rows;
  applyImportFilter();
}

function applyImportFilter() {
  const rows = window._importRows || [];
  const filterText = (document.getElementById('import-filter').value || '').toLowerCase();
  const nonOkOnly = document.getElementById('import-stubs-only').checked;
  const tbody = document.getElementById('import-tbody');

  const filtered = rows.filter(r => {
    if (nonOkOnly && r.kind === 'implemented') return false;
    if (filterText) {
      return r.dll.toLowerCase().includes(filterText) || r.name.toLowerCase().includes(filterText);
    }
    return true;
  });

  tbody.innerHTML = filtered.map(r =>
    `<tr>
      <td>${esc(r.dll)}</td>
      <td>${esc(r.name)}</td>
      <td class="${importStatusClass(r.kind)}">${esc(importStatusLabel(r.kind))}</td>
    </tr>`
  ).join('') || '<tr><td colspan="3" class="placeholder">No matching imports</td></tr>';
}

function bindImportsUi() {
  document.getElementById('import-filter').addEventListener('input', applyImportFilter);
  document.getElementById('import-stubs-only').addEventListener('change', applyImportFilter);
}
