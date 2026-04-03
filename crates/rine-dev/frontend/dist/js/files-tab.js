// Files tab logic.

function renderFilesTable() {
  const tbody = document.getElementById('file-tbody');
  const filterText = (document.getElementById('file-filter').value || '').toLowerCase();
  const hideClosed = document.getElementById('file-hide-closed').checked;

  const filtered = state.handles.filter(h => {
    if (h.kind !== 'File') return false;
    if (hideClosed && h.closed) return false;
    if (filterText) {
      return h.detail.toLowerCase().includes(filterText) || String(h.handle).includes(filterText);
    }
    return true;
  });

  if (filtered.length === 0) {
    tbody.innerHTML = '<tr><td colspan="3" class="placeholder">No matching files</td></tr>';
    return;
  }

  tbody.innerHTML = filtered.map(h =>
    `<tr class="${h.closed ? 'row-closed' : ''}">
      <td>${hex(h.handle)}</td>
      <td>${esc(h.detail)}</td>
      <td class="${h.closed ? 'status-closed' : 'status-open'}">${h.closed ? 'Closed' : 'Open'}</td>
    </tr>`
  ).join('');
}

function bindFilesUi() {
  document.getElementById('file-filter').addEventListener('input', renderFilesTable);
  document.getElementById('file-hide-closed').addEventListener('change', renderFilesTable);
}
