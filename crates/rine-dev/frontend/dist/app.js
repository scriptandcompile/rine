// rine-dev dashboard — frontend logic
const { listen, invoke } = (function() {
  // Tauri v2 IPC
  const tauri = window.__TAURI__;
  const event = tauri.event;
  const core  = tauri.core;
  return {
    listen: event.listen.bind(event),
    invoke: core.invoke.bind(core),
  };
})();

// ── State ──────────────────────────────────────────
let state = { pe: null, config: null, imports: null, exited: null, exitCode: null, stdout: '', stderr: '', handles: [], threads: [], tls_slots: [] };
let events = [];
let startTime = Date.now();

// ── Tabs ───────────────────────────────────────────
document.querySelectorAll('.tab').forEach(btn => {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.tab').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(s => s.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById('tab-' + btn.dataset.tab).classList.add('active');
  });
});

// ── Hex helper ─────────────────────────────────────
function hex(n) { return '0x' + BigInt(n).toString(16).toUpperCase(); }

// ── Rendering ─────────────────────────────────────
function renderPeInfo(pe) {
  const el = document.getElementById('pe-info');
  el.innerHTML = [
    kv('Executable', pe.exe_path),
    kv('Image Base', hex(pe.image_base)),
    kv('Image Size', hex(pe.image_size)),
    kv('Entry RVA', hex(pe.entry_rva)),
    kv('Relocation Delta', pe.relocation_delta >= 0
      ? '+' + hex(pe.relocation_delta)
      : '-' + hex(-pe.relocation_delta)),
    kv('Sections', pe.sections.length),
  ].join('');
}

function renderConfigInfo(cfg) {
  const el = document.getElementById('config-info');
  let html = [
    kv('Config File', cfg.config_path),
    kv('Windows Version', cfg.windows_version),
  ].join('');

  if (cfg.environment_overrides.length > 0) {
    html += '<div style="margin-top:8px"><span class="kv-key">Environment Overrides:</span></div>';
    for (const [k, v] of cfg.environment_overrides) {
      html += `<div class="kv" style="padding-left:12px"><span class="kv-key">${esc(k)}</span><span class="kv-val">${esc(v)}</span></div>`;
    }
  } else {
    html += kv('Environment', 'none');
  }
  el.innerHTML = html;
}

function renderImportSummary(imp) {
  const el = document.getElementById('import-summary');
  const total = imp.total_resolved + imp.total_stubbed;
  const pctResolved = total > 0 ? (imp.total_resolved / total * 100) : 100;
  const pctStubbed  = total > 0 ? (imp.total_stubbed / total * 100) : 0;

  el.innerHTML = [
    kv('Total', total),
    kv('Resolved', imp.total_resolved),
    kv('Stubbed', imp.total_stubbed),
    `<div class="import-bar">`,
    `  <div class="import-bar-resolved" style="width:${pctResolved}%"></div>`,
    `  <div class="import-bar-stubbed" style="width:${pctStubbed}%"></div>`,
    `</div>`,
    `<div class="import-legend">`,
    `  <span class="legend-resolved">Resolved (${imp.total_resolved})</span>`,
    `  <span class="legend-stubbed">Stubbed (${imp.total_stubbed})</span>`,
    `</div>`,
  ].join('');

  // Also update the imports table
  renderImportTable(imp);
}

function renderSections(sections) {
  const el = document.getElementById('sections-info');
  let html = '<table class="sections-table"><thead><tr>' +
    '<th>Name</th><th>VirtualAddress</th><th>VirtualSize</th><th>Characteristics</th>' +
    '</tr></thead><tbody>';
  for (const s of sections) {
    html += `<tr>
      <td>${esc(s.name)}</td>
      <td>${hex(s.virtual_address)}</td>
      <td>${hex(s.virtual_size)}</td>
      <td>${hex(s.characteristics)}</td>
    </tr>`;
  }
  html += '</tbody></table>';
  el.innerHTML = html;
}

function renderImportTable(imp) {
  const tbody = document.getElementById('import-tbody');
  let rows = [];
  for (const dll of imp.summaries) {
    for (const name of dll.resolved_names) {
      rows.push({ dll: dll.dll_name, name, stub: false });
    }
    for (const name of dll.stubbed_names) {
      rows.push({ dll: dll.dll_name, name, stub: true });
    }
  }

  // Sort: stubs first, then alphabetical
  rows.sort((a, b) => {
    if (a.stub !== b.stub) return a.stub ? -1 : 1;
    return a.dll.localeCompare(b.dll) || a.name.localeCompare(b.name);
  });

  // Store for filtering
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
      return r.dll.toLowerCase().includes(filterText) ||
             r.name.toLowerCase().includes(filterText);
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

// ── Handles table ──────────────────────────────────
function renderHandlesTable() {
  const tbody = document.getElementById('handle-tbody');
  const filterText = (document.getElementById('handle-filter').value || '').toLowerCase();
  const hideClosed = document.getElementById('handle-hide-closed').checked;

  const filtered = state.handles.filter(h => {
    if (hideClosed && h.closed) return false;
    if (filterText) {
      return h.kind.toLowerCase().includes(filterText) ||
             h.detail.toLowerCase().includes(filterText) ||
             String(h.handle).includes(filterText);
    }
    return true;
  });

  if (filtered.length === 0) {
    tbody.innerHTML = '<tr><td colspan="4" class="placeholder">No matching handles</td></tr>';
    return;
  }

  tbody.innerHTML = filtered.map(h =>
    `<tr class="${h.closed ? 'row-closed' : ''}">
      <td>${hex(h.handle)}</td>
      <td>${esc(h.kind)}</td>
      <td>${esc(h.detail)}</td>
      <td class="${h.closed ? 'status-closed' : 'status-open'}">${h.closed ? 'Closed' : 'Open'}</td>
    </tr>`
  ).join('');
}

// ── Threads table ──────────────────────────────────
function renderThreadsTable() {
  const tbody = document.getElementById('thread-tbody');
  const filterText = (document.getElementById('thread-filter').value || '').toLowerCase();

  const filtered = state.threads.filter(t => {
    if (filterText) {
      return String(t.thread_id).includes(filterText) ||
             hex(t.entry_point).toLowerCase().includes(filterText);
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

function addEventEntry(event) {
  const log = document.getElementById('event-log');
  const elapsed = ((Date.now() - startTime) / 1000).toFixed(3);
  const div = document.createElement('div');
  div.className = 'event-entry';

  let detail = '';
  switch (event.type) {
    case 'PeLoaded':
      detail = `base=${hex(event.image_base)}  size=${hex(event.image_size)}  sections=${event.sections.length}`;
      break;
    case 'ConfigLoaded':
      detail = `version=${event.windows_version}  overrides=${event.environment_overrides.length}`;
      break;
    case 'ImportsResolved':
      detail = `resolved=${event.total_resolved}  stubbed=${event.total_stubbed}`;
      break;
    case 'ProcessExited':
      detail = `exit_code=${event.exit_code}`;
      break;
    case 'HandleCreated':
      detail = `handle=${hex(event.handle)}  type=${event.kind}  detail=${event.detail}`;
      break;
    case 'HandleClosed':
      detail = `handle=${hex(event.handle)}`;
      break;
    case 'ThreadCreated':
      detail = `tid=${event.thread_id}  handle=${hex(event.handle)}  entry=${hex(event.entry_point)}`;
      break;
    case 'ThreadExited':
      detail = `tid=${event.thread_id}  exit_code=${event.exit_code}`;
      break;
    case 'TlsAllocated':
      detail = `index=${event.index}`;
      break;
    case 'TlsFreed':
      detail = `index=${event.index}`;
      break;
    default:
      detail = JSON.stringify(event);
  }

  div.innerHTML = `<span class="event-ts">[${elapsed}s]</span><span class="event-type">${esc(event.type)}</span> ${esc(detail)}`;

  const filterText = (document.getElementById('event-filter').value || '').toLowerCase();
  if (filterText && !div.textContent.toLowerCase().includes(filterText)) {
    div.style.display = 'none';
  }

  log.appendChild(div);

  if (document.getElementById('event-autoscroll').checked) {
    log.scrollTop = log.scrollHeight;
  }
}

function updateStatusBar() {
  const el = document.getElementById('stat-status');
  if (state.exited != null) {
    el.textContent = state.exitCode != null
      ? (state.exitCode === 0 ? 'Status: exited - success' : `Status: exited - code ${state.exitCode}`)
      : 'Status: exited';
  } else if (state.pe) {
    el.textContent = 'Status: running';
  } else {
    el.textContent = 'Status: waiting';
  }

  if (state.imports) {
    document.getElementById('stat-imports').textContent =
      `Imports: ${state.imports.total_resolved} resolved, ${state.imports.total_stubbed} stubbed`;
  }
  if (state.pe) {
    document.getElementById('stat-sections').textContent =
      `Sections: ${state.pe.sections.length}`;
  }

  const openHandles = state.handles.filter(h => !h.closed).length;
  document.getElementById('stat-handles').textContent =
    `Handles: ${openHandles} open / ${state.handles.length} total`;

  const runningThreads = state.threads.filter(t => t.exit_code == null).length;
  document.getElementById('stat-threads').textContent =
    `Threads: ${runningThreads} running / ${state.threads.length} total`;
}

function renderOutput(stream, text) {
  const el = document.getElementById('output-' + stream);
  el.innerHTML = typeof ansiToHtml === 'function' ? ansiToHtml(text) : esc(text);
  el.scrollTop = el.scrollHeight;
}

function updateStatusBadge() {
  const badge = document.getElementById('status-badge');
  if (state.exited != null) {
    badge.textContent = state.exitCode != null
      ? (state.exitCode === 0 ? 'exited - success' : `exited - code ${state.exitCode}`)
      : 'exited';
    badge.className = state.exitCode === 0 ? 'badge badge-exited-ok' : 'badge badge-exited';
  } else if (state.pe) {
    badge.textContent = 'running';
    badge.className = 'badge badge-running';
  }
}

// ── Helpers ────────────────────────────────────────
function kv(key, val) {
  return `<div class="kv"><span class="kv-key">${esc(String(key))}</span><span class="kv-val">${esc(String(val))}</span></div>`;
}
function esc(s) {
  const d = document.createElement('div');
  d.textContent = s;
  return d.innerHTML;
}

// ── Event handlers ─────────────────────────────────
function handleEvent(event) {
  events.push(event);
  addEventEntry(event);

  switch (event.type) {
    case 'PeLoaded':
      state.pe = event;
      renderPeInfo(event);
      renderSections(event.sections);
      break;
    case 'ConfigLoaded':
      state.config = event;
      renderConfigInfo(event);
      break;
    case 'ImportsResolved':
      state.imports = event;
      renderImportSummary(event);
      break;
    case 'ProcessExited':
      state.exited = true;
      state.exitCode = event.exit_code;
      break;
    case 'HandleCreated':
      state.handles.push({ handle: event.handle, kind: event.kind, detail: event.detail, closed: false });
      renderHandlesTable();
      break;
    case 'HandleClosed':
      { const h = state.handles.find(h => h.handle === event.handle && !h.closed);
        if (h) h.closed = true; }
      renderHandlesTable();
      break;
    case 'ThreadCreated':
      state.threads.push({ handle: event.handle, thread_id: event.thread_id, entry_point: event.entry_point, exit_code: null });
      renderThreadsTable();
      break;
    case 'ThreadExited':
      { const t = state.threads.find(t => t.thread_id === event.thread_id && t.exit_code == null);
        if (t) t.exit_code = event.exit_code; }
      renderThreadsTable();
      break;
    case 'TlsAllocated':
      state.tls_slots.push(event.index);
      break;
    case 'TlsFreed':
      state.tls_slots = state.tls_slots.filter(i => i !== event.index);
      break;
    case 'OutputData':
      if (event.stream === 'Stdout') {
        state.stdout += event.data;
        renderOutput('stdout', state.stdout);
      } else {
        state.stderr += event.data;
        renderOutput('stderr', state.stderr);
      }
      break;
  }

  updateStatusBar();
  updateStatusBadge();
}

// ── Wire up ────────────────────────────────────────
listen('dev-event', (e) => {
  handleEvent(e.payload);
});

listen('rine-disconnected', () => {
  if (state.exited == null) {
    state.exited = true;
    updateStatusBar();
    updateStatusBadge();
  }
});

// Output sub-tab switching
for (const btn of document.querySelectorAll('.output-tab')) {
  btn.addEventListener('click', () => {
    document.querySelectorAll('.output-tab').forEach(b => b.classList.remove('active'));
    document.querySelectorAll('.output-pane').forEach(p => p.classList.remove('active'));
    btn.classList.add('active');
    document.getElementById('output-' + btn.dataset.output).classList.add('active');
  });
}

// Filter listeners
document.getElementById('import-filter').addEventListener('input', applyImportFilter);
document.getElementById('import-stubs-only').addEventListener('change', applyImportFilter);
document.getElementById('handle-filter').addEventListener('input', renderHandlesTable);
document.getElementById('handle-hide-closed').addEventListener('change', renderHandlesTable);
document.getElementById('thread-filter').addEventListener('input', renderThreadsTable);
document.getElementById('event-filter').addEventListener('input', () => {
  const filterText = document.getElementById('event-filter').value.toLowerCase();
  document.querySelectorAll('#event-log .event-entry').forEach(div => {
    div.style.display = filterText && !div.textContent.toLowerCase().includes(filterText)
      ? 'none' : '';
  });
});

// On load, try to get existing state (in case we reconnect)
invoke('get_state').then(snap => {
  if (snap.pe) { state.pe = snap.pe; renderPeInfo(snap.pe); renderSections(snap.pe.sections); }
  if (snap.config) { state.config = snap.config; renderConfigInfo(snap.config); }
  if (snap.imports) { state.imports = snap.imports; renderImportSummary(snap.imports); }
  if (snap.exited != null) {
    state.exited = true;
    state.exitCode = snap.exited === -1 ? null : snap.exited;
  }
  if (snap.stdout) { state.stdout = snap.stdout; renderOutput('stdout', state.stdout); }
  if (snap.stderr) { state.stderr = snap.stderr; renderOutput('stderr', state.stderr); }
  if (snap.handles && snap.handles.length) { state.handles = snap.handles; renderHandlesTable(); }
  if (snap.threads && snap.threads.length) { state.threads = snap.threads; renderThreadsTable(); }
  if (snap.tls_slots && snap.tls_slots.length) { state.tls_slots = snap.tls_slots; }
  updateStatusBar();
  updateStatusBadge();
}).catch(() => {});
