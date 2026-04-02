// Mutexes tab logic.

function renderMutexesTable() {
  const tbody = document.getElementById('mutex-tbody');
  const filterText = (document.getElementById('mutex-filter').value || '').toLowerCase();
  const hideClosed = document.getElementById('mutex-hide-closed').checked;

  const filtered = state.handles.filter(h => {
    if (h.kind !== 'Mutex') return false;
    if (hideClosed && h.closed) return false;
    if (filterText) {
      return h.detail.toLowerCase().includes(filterText) || String(h.handle).includes(filterText);
    }
    return true;
  });

  if (filtered.length === 0) {
    tbody.innerHTML = '<tr><td colspan="3" class="placeholder">No matching mutexes</td></tr>';
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

function bindMutexesUi() {
  document.getElementById('mutex-filter').addEventListener('input', renderMutexesTable);
  document.getElementById('mutex-hide-closed').addEventListener('change', renderMutexesTable);
}
