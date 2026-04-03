// Dialogs tab logic.

function countPendingDialogs(calls) {
  const pendingByApi = new Map();
  for (const call of calls) {
    const key = call.api || 'unknown';
    const current = pendingByApi.get(key) || 0;
    if (call.phase === 'opened') {
      pendingByApi.set(key, current + 1);
    } else if (call.phase === 'result') {
      pendingByApi.set(key, Math.max(0, current - 1));
    }
  }

  let total = 0;
  for (const count of pendingByApi.values()) {
    total += count;
  }
  return total;
}

function renderPendingDialogCount(calls) {
  const el = document.getElementById('dialog-pending-count');
  const pending = countPendingDialogs(calls);
  el.textContent = `Pending dialogs: ${pending}`;
}

function renderDialogTable() {
  const tbody = document.getElementById('dialog-tbody');
  const filterText = (document.getElementById('dialog-filter').value || '').toLowerCase();
  const failedOnly = document.getElementById('dialog-failed-only').checked;

  const calls = state.dialog_calls || [];
  renderPendingDialogCount(calls);

  const filtered = calls.filter(call => {
    if (failedOnly && call.success !== false) return false;
    if (!filterText) return true;

    const haystack = [
      call.api,
      call.phase,
      call.theme,
      call.windows_theme,
      call.selected_path || '',
      String(call.error_code ?? ''),
      call.error_code == null ? '' : '0x' + Number(call.error_code).toString(16),
      call.success == null ? 'opened' : (call.success ? 'success' : 'failure'),
    ]
      .join(' ')
      .toLowerCase();

    return haystack.includes(filterText);
  });

  if (filtered.length === 0) {
    tbody.innerHTML = '<tr><td colspan="7" class="placeholder">No matching dialog calls</td></tr>';
    return;
  }

  tbody.innerHTML = filtered
    .map(call => {
      const statusClass = call.success == null ? '' : (call.success ? 'status-open' : 'status-exited');
      const status = call.success == null ? 'Opened' : (call.success ? 'Success' : 'Failure');
      const errHex = call.error_code == null ? '' : ('0x' + Number(call.error_code).toString(16).toUpperCase());
      return `<tr>
        <td>${esc(call.api)}</td>
        <td>${esc(call.phase)}</td>
        <td>${esc(call.theme)}</td>
        <td>${esc(call.windows_theme)}</td>
        <td class="${statusClass}">${status}</td>
        <td>${esc(errHex)}</td>
        <td>${esc(call.selected_path || '')}</td>
      </tr>`;
    })
    .join('');
}

function bindDialogUi() {
  document.getElementById('dialog-filter').addEventListener('input', renderDialogTable);
  document.getElementById('dialog-failed-only').addEventListener('change', renderDialogTable);
}
