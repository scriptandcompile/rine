// GDI tab logic.

const GDI_HANDLE_KINDS = new Set(['GdiDc', 'GdiBitmap', 'GdiBrush', 'GdiPen']);

function isGdiHandleKind(kind) {
  return GDI_HANDLE_KINDS.has(kind);
}

function formatGdiDetail(detail) {
  try {
    const parsed = JSON.parse(detail || '{}');
    return Object.entries(parsed)
      .map(([k, v]) => `${k}=${String(v)}`)
      .join('  ');
  } catch {
    return detail || '';
  }
}

function renderGdiTable() {
  const tbody = document.getElementById('gdi-tbody');
  const filterText = (document.getElementById('gdi-filter').value || '').toLowerCase();
  const hideClosed = document.getElementById('gdi-hide-closed').checked;

  const filtered = state.handles.filter(h => {
    if (!isGdiHandleKind(h.kind)) return false;
    if (hideClosed && h.closed) return false;

    if (!filterText) return true;

    const detailText = formatGdiDetail(h.detail).toLowerCase();
    return (
      h.kind.toLowerCase().includes(filterText)
      || detailText.includes(filterText)
      || String(h.handle).includes(filterText)
      || hex(h.handle).toLowerCase().includes(filterText)
    );
  });

  if (filtered.length === 0) {
    tbody.innerHTML = '<tr><td colspan="4" class="placeholder">No matching GDI objects</td></tr>';
    return;
  }

  tbody.innerHTML = filtered.map(h =>
    `<tr class="${h.closed ? 'row-closed' : ''}">
      <td>${hex(h.handle)}</td>
      <td>${esc(h.kind)}</td>
      <td>${esc(formatGdiDetail(h.detail))}</td>
      <td class="${h.closed ? 'status-closed' : 'status-open'}">${h.closed ? 'Closed' : 'Open'}</td>
    </tr>`
  ).join('');
}

function bindGdiUi() {
  document.getElementById('gdi-filter').addEventListener('input', renderGdiTable);
  document.getElementById('gdi-hide-closed').addEventListener('change', renderGdiTable);
}
