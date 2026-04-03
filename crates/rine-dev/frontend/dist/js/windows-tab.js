// Windows tab logic.

function escapeWindowHtml(text) {
  const map = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#039;' };
  return String(text).replace(/[&<>"']/g, m => map[m]);
}

function renderWindowTree() {
  const treeWrap = document.querySelector('#tab-windows .tree-wrap');
  const filterText = (document.getElementById('window-filter')?.value || '').trim().toLowerCase();
  const hideDestroyed = !!document.getElementById('window-hide-destroyed')?.checked;

  if (state.windows.length === 0) {
    treeWrap.innerHTML = '<div class="placeholder">No windows created yet</div>';
    return;
  }

  const windowsByParent = new Map();
  for (const w of state.windows) {
    const parent = w.parent || 0;
    if (!windowsByParent.has(parent)) windowsByParent.set(parent, []);
    windowsByParent.get(parent).push(w);
  }

  const matchesSelf = (w) => {
    if (hideDestroyed && w.destroyed) return false;
    if (!filterText) return true;
    return (
      String(w.hwnd).toLowerCase().includes(filterText)
      || hex(w.hwnd).toLowerCase().includes(filterText)
      || String(w.title || '').toLowerCase().includes(filterText)
      || String(w.class_name || '').toLowerCase().includes(filterText)
    );
  };

  const visibleCache = new Map();
  function isVisible(w) {
    if (visibleCache.has(w.hwnd)) return visibleCache.get(w.hwnd);
    const children = windowsByParent.get(w.hwnd) || [];
    const visible = matchesSelf(w) || children.some(isVisible);
    visibleCache.set(w.hwnd, visible);
    return visible;
  }

  const rootWindows = (windowsByParent.get(0) || []).filter(isVisible);

  if (rootWindows.length === 0) {
    treeWrap.innerHTML = '<div class="placeholder">No matching windows</div>';
    return;
  }

  function buildTreeNode(window) {
    const isExpanded = expandedWindows.has(window.hwnd);
    const children = (windowsByParent.get(window.hwnd) || []).filter(isVisible);
    const hasChildren = children.length > 0;
    const statusClass = window.destroyed ? 'status-exited' : 'status-running';

    let html = `<div class="tree-node">`;

    if (hasChildren) {
      html += `<span class="tree-expand" data-hwnd="${window.hwnd}">${isExpanded ? '▼' : '▶'}</span>`;
    } else {
      html += `<span class="tree-expand-space"></span>`;
    }

    html += `<span class="tree-hwnd">${hex(window.hwnd)}</span>`;
    html += `<span class="tree-title">${escapeWindowHtml(window.title || '(no title)')}</span>`;
    html += `<span class="tree-class">${escapeWindowHtml(window.class_name || '(no class)')}</span>`;
    html += `<span class="tree-status ${statusClass}">${window.destroyed ? 'Destroyed' : 'Active'}</span>`;
    html += `</div>`;

    if (hasChildren && isExpanded) {
      html += `<div class="tree-children">`;
      children.forEach(child => { html += buildTreeNode(child); });
      html += `</div>`;
    } else if (hasChildren) {
      html += `<div class="tree-children collapsed">`;
      children.forEach(child => { html += buildTreeNode(child); });
      html += `</div>`;
    }

    return html;
  }

  treeWrap.innerHTML = rootWindows.map(w => buildTreeNode(w)).join('');

  treeWrap.querySelectorAll('.tree-expand').forEach(btn => {
    btn.addEventListener('click', () => {
      const hwnd = parseInt(btn.dataset.hwnd);
      if (expandedWindows.has(hwnd)) {
        expandedWindows.delete(hwnd);
      } else {
        expandedWindows.add(hwnd);
      }
      renderWindowTree();
    });
  });
}

function hydrateWindowsFromHandles(handles) {
  state.windows = handles
    .filter(h => h.kind === 'Window')
    .map(h => {
      try {
        const winInfo = JSON.parse(h.detail || '{}');
        return {
          hwnd: winInfo.hwnd || h.handle,
          title: winInfo.title || '',
          class_name: winInfo.class_name || '',
          parent: winInfo.parent || 0,
          destroyed: !!h.closed,
        };
      } catch {
        return {
          hwnd: h.handle,
          title: '',
          class_name: '',
          parent: 0,
          destroyed: !!h.closed,
        };
      }
    });
}

function handleWindowCreatedFromHandleEvent(event) {
  try {
    const winInfo = JSON.parse(event.detail);
    state.windows.push({
      hwnd: winInfo.hwnd || event.handle,
      title: winInfo.title || '',
      class_name: winInfo.class_name || '',
      parent: winInfo.parent || 0,
      destroyed: false,
    });
    renderWindowTree();
  } catch (e) {
    console.warn('Failed to parse window info:', event.detail, e);
  }
}

function handleWindowClosedFromHandleEvent(event) {
  const w = state.windows.find(w => w.hwnd === event.handle && !w.destroyed);
  if (w) {
    w.destroyed = true;
    renderWindowTree();
  }
}

function bindWindowsUi() {
  document.getElementById('window-filter').addEventListener('input', renderWindowTree);
  document.getElementById('window-hide-destroyed').addEventListener('change', renderWindowTree);
}
