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
let state = {
  pe: null,
  config: null,
  imports: null,
  dll_registry_metrics: null,
  exited: null,
  exitCode: null,
  stdout: '',
  stderr: '',
  handles: [],
  threads: [],
  tls_slots: [],
  windows: [],
  dialog_calls: [],
  memory_regions: [],
  memory_current_usage: 0,
  memory_peak_usage: 0,
  memory_total_allocated: 0,
  memory_total_freed: 0,
  memory_snapshot: null,
  memory_snapshot_meta: null,
};
let events = [];
let startTime = Date.now();
let expandedWindows = new Set(); // Track expanded window nodes

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
    kv('Architecture', pe.architecture || 'unknown'),
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
  const total = imp.total_resolved + imp.total_partial + imp.total_stubbed + imp.total_unimplemented;
  const pctResolved = total > 0 ? (imp.total_resolved / total * 100) : 100;
  const pctPartial = total > 0 ? (imp.total_partial / total * 100) : 0;
  const pctStubbed = total > 0 ? (imp.total_stubbed / total * 100) : 0;
  const pctUnimplemented = total > 0 ? (imp.total_unimplemented / total * 100) : 0;

  el.innerHTML = [
    kv('Total', total),
    kv('OK', imp.total_resolved),
    kv('Partial', imp.total_partial),
    kv('Stubbed', imp.total_stubbed),
    kv('Not Implemented', imp.total_unimplemented),
    `<div class="import-bar">`,
    `  <div class="import-bar-resolved" style="width:${pctResolved}%"></div>`,
    `  <div class="import-bar-partial" style="width:${pctPartial}%"></div>`,
    `  <div class="import-bar-stubbed" style="width:${pctStubbed}%"></div>`,
    `  <div class="import-bar-unimplemented" style="width:${pctUnimplemented}%"></div>`,
    `</div>`,
    `<div class="import-legend">`,
    `  <span class="legend-resolved">OK (${imp.total_resolved})</span>`,
    `  <span class="legend-partial">Partial (${imp.total_partial})</span>`,
    `  <span class="legend-stubbed">Stubbed (${imp.total_stubbed})</span>`,
    `  <span class="legend-unimplemented">Not Implemented (${imp.total_unimplemented})</span>`,
    `</div>`,
  ].join('');

  // Also update the imports table
  renderImportTable(imp);
}

function renderRegistryMetrics(metrics) {
  const el = document.getElementById('registry-metrics');
  const totalLookups = metrics.name_lookups + metrics.ordinal_lookups;
  el.innerHTML = [
    kv('Registered DLLs', metrics.registered_dlls),
    kv('Loaded DLLs', metrics.loaded_dlls),
    kv('Lazy Loads', metrics.lazy_loads),
    kv('Cache Hits', metrics.cache_hits),
    kv('Name Lookups', metrics.name_lookups),
    kv('Ordinal Lookups', metrics.ordinal_lookups),
    kv('Total Lookups', totalLookups),
  ].join('');
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

function addEventEntry(event) {
  const log = document.getElementById('event-log');
  const elapsed = ((Date.now() - startTime) / 1000).toFixed(3);
  const div = document.createElement('div');
  div.className = 'event-entry';

  let detail = '';
  switch (event.type) {
    case 'PeLoaded':
      detail = `arch=${event.architecture || 'unknown'}  base=${hex(event.image_base)}  size=${hex(event.image_size)}  sections=${event.sections.length}`;
      break;
    case 'ConfigLoaded':
      detail = `version=${event.windows_version}  overrides=${event.environment_overrides.length}`;
      break;
    case 'ImportsResolved':
      detail = `ok=${event.total_resolved}  partial=${event.total_partial}  stubbed=${event.total_stubbed}  unimplemented=${event.total_unimplemented}`;
      break;
    case 'DllRegistryMetrics':
      detail = `registered=${event.registered_dlls}  loaded=${event.loaded_dlls}  lazy_loads=${event.lazy_loads}  cache_hits=${event.cache_hits}  lookups=${event.name_lookups + event.ordinal_lookups}`;
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
    case 'MemoryAllocated':
      detail = `address=${hex(event.address)}  size=${formatBytesWithParens(event.size)}  source=${event.source}`;
      break;
    case 'MemoryFreed':
      detail = `address=${hex(event.address)}  size=${formatBytesWithParens(event.size)}  source=${event.source}`;
      break;
    case 'MemorySnapshotReady':
      detail = `regions=${event.region_count}  bytes=${formatBytesWithParens(event.total_bytes)}  json=${event.json_path}`;
      break;
    case 'DialogOpened':
      detail = `api=${event.api}  theme=${event.theme}  backend=${event.native_backend}  windows_style=${event.windows_theme}`;
      break;
    case 'DialogResult':
      detail = `api=${event.api}  theme=${event.theme}  backend=${event.native_backend}  windows_style=${event.windows_theme}  success=${event.success}  error=0x${Number(event.error_code).toString(16).toUpperCase()}${event.selected_path ? `  path=${event.selected_path}` : ''}`;
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
      ? (state.exitCode === 0 ? 'Status: exit - success (0)' : `Status: exited - code ${state.exitCode}`)
      : 'Status: exited';
  } else if (state.pe) {
    el.textContent = 'Status: running';
  } else {
    el.textContent = 'Status: waiting for rine';
  }

  if (state.imports) {
    document.getElementById('stat-imports').textContent =
      `Imports: ${state.imports.total_resolved} ok, ${state.imports.total_partial} partial, ${state.imports.total_stubbed} stubbed, ${state.imports.total_unimplemented} not implemented`;
  }
  if (state.dll_registry_metrics) {
    document.getElementById('stat-sections').textContent =
      state.pe
        ? `Sections: ${state.pe.sections.length}  DLLs: ${state.dll_registry_metrics.loaded_dlls}/${state.dll_registry_metrics.registered_dlls} loaded`
        : `DLLs: ${state.dll_registry_metrics.loaded_dlls}/${state.dll_registry_metrics.registered_dlls} loaded`;
  } else if (state.pe) {
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
      ? (state.exitCode === 0 ? 'exit - success (0)' : `exited - code ${state.exitCode}`)
      : 'exited';
    badge.className = state.exitCode === 0 ? 'badge badge-exited-ok' : 'badge badge-exited';
  } else if (state.pe) {
    badge.textContent = 'running';
    badge.className = 'badge badge-running';
  } else {
    badge.textContent = 'waiting for rine…';
    badge.className = 'badge badge-waiting';
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
    case 'DllRegistryMetrics':
      state.dll_registry_metrics = event;
      renderRegistryMetrics(event);
      break;
    case 'ProcessExited':
      state.exited = true;
      state.exitCode = event.exit_code;
      break;
    case 'HandleCreated':
      state.handles.push({ handle: event.handle, kind: event.kind, detail: event.detail, closed: false });
      renderFilesTable();
      renderMutexesTable();
      renderGdiTable();
      if (event.kind === 'Window') {
        handleWindowCreatedFromHandleEvent(event);
      }
      break;
    case 'HandleClosed': {
      const h = state.handles.find(h => h.handle === event.handle && !h.closed);
      if (h) h.closed = true;
      renderFilesTable();
      renderMutexesTable();
      renderGdiTable();
      if (h && h.kind === 'Window') {
        handleWindowClosedFromHandleEvent(event);
      }
      break;
    }
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
    case 'MemoryAllocated':
    case 'MemoryFreed':
    case 'MemorySnapshotReady':
      handleMemoryEvent(event);
      break;
    case 'DialogOpened':
      state.dialog_calls.push({
        phase: 'opened',
        api: event.api,
        theme: event.theme,
        native_backend: event.native_backend,
        windows_theme: event.windows_theme,
        success: null,
        error_code: null,
        selected_path: null,
      });
      renderDialogTable();
      break;
    case 'DialogResult':
      state.dialog_calls.push({
        phase: 'result',
        api: event.api,
        theme: event.theme,
        native_backend: event.native_backend,
        windows_theme: event.windows_theme,
        success: event.success,
        error_code: event.error_code,
        selected_path: event.selected_path,
      });
      renderDialogTable();
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
  renderFinalMemoryMapping();
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
    renderFinalMemoryMapping();
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
bindImportsUi();
bindFilesUi();
bindThreadsUi();
bindMemoryUi();
bindMutexesUi();
bindWindowsUi();
bindGdiUi();
bindDialogUi();
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
  if (snap.dll_registry_metrics) {
    state.dll_registry_metrics = snap.dll_registry_metrics;
    renderRegistryMetrics(snap.dll_registry_metrics);
  }
  if (snap.exited != null) {
    state.exited = true;
    state.exitCode = snap.exited === -1 ? null : snap.exited;
  }
  if (snap.stdout) { state.stdout = snap.stdout; renderOutput('stdout',state.stdout); }
  if (snap.stderr) { state.stderr = snap.stderr; renderOutput('stderr', state.stderr); }
  if (snap.handles && snap.handles.length) { 
    state.handles = snap.handles;
    renderFilesTable();
    renderMutexesTable();
    renderGdiTable();
    hydrateWindowsFromHandles(snap.handles);
    renderWindowTree();
  }
  if (snap.threads && snap.threads.length) { state.threads = snap.threads; renderThreadsTable(); }
  if (snap.tls_slots && snap.tls_slots.length) { state.tls_slots = snap.tls_slots; }
  if (snap.dialog_calls && snap.dialog_calls.length) {
    state.dialog_calls = snap.dialog_calls;
    renderDialogTable();
  }
  hydrateMemoryState(snap);
  renderMemorySummary();
  renderMemoryTable();
  updateStatusBar();
  updateStatusBadge();
  renderFinalMemoryMapping();
}).catch(() => {});
