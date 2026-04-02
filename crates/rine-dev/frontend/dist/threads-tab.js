// Threads tab logic.

function renderThreadsTable() {
  const tbody = document.getElementById('thread-tbody');
  const filterText = (document.getElementById('thread-filter').value || '').toLowerCase();

  const filtered = state.threads.filter(t => {
    if (filterText) {
      return String(t.thread_id).includes(filterText) || hex(t.entry_point).toLowerCase().includes(filterText);
    }
    return true;
  });

  if (filtered.length === 0) {
    tbody.innerHTML = '<tr><td colspan="5" class="placeholder">No matching threads</td></tr>';
    return;
  }

  tbody.innerHTML = filtered.map(t => {
    const exited = t.exit_code != null;
    return `<tr>
      <td>${t.thread_id}</td>
      <td>${hex(t.handle)}</td>
      <td>${hex(t.entry_point)}</td>
      <td class="${exited ? 'status-exited' : 'status-running'}">${exited ? 'Exited' : 'Running'}</td>
      <td>${exited ? t.exit_code : '—'}</td>
    </tr>`;
  }).join('');
}

function bindThreadsUi() {
  document.getElementById('thread-filter').addEventListener('input', renderThreadsTable);
}
