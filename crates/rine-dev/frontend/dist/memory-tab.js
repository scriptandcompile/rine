// Memory tab logic (formatting, rendering, export, and state hydration).

function toBigInt(n) {
  try {
    return typeof n === 'bigint' ? n : BigInt(n);
  } catch {
    return 0n;
  }
}

function formatBytesHuman(n) {
  const units = ['bytes', 'Kb', 'Mb', 'Gb', 'Tb', 'Pb'];
  let value = Number(toBigInt(n));
  if (!Number.isFinite(value) || value < 0) value = 0;

  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  if (unitIndex === 0) {
    return `${Math.round(value)} bytes`;
  }

  const digits = value >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${units[unitIndex]}`;
}

function formatBytesGrouped(n) {
  const bytes = toBigInt(n);
  return bytes.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',');
}

function formatBytesDetailed(n) {
  const bytes = toBigInt(n);
  return `${formatBytesGrouped(bytes)} bytes, ${formatBytesHuman(bytes)}`;
}

function formatBytesWithParens(n) {
  const bytes = toBigInt(n);
  return `${formatBytesGrouped(bytes)} bytes (${formatBytesHuman(bytes)})`;
}

function buildMemoryDumpData() {
  const status = state.exited == null
    ? 'running'
    : (state.exitCode == null ? 'crashed_or_disconnected' : 'exited');

  const regions = state.memory_regions
    .slice()
    .sort((a, b) => Number(a.address) - Number(b.address))
    .map(r => ({
      address: r.address,
      address_hex: hex(r.address),
      size: r.size,
      size_human: formatBytesHuman(r.size),
      source: r.source,
      status: r.freed ? 'Freed' : 'Active',
      freed: !!r.freed,
    }));

  return {
    generated_at: new Date().toISOString(),
    process_status: status,
    process_exit_code: state.exitCode,
    memory_summary: {
      current: {
        value: state.memory_current_usage,
        hex: hex(state.memory_current_usage),
        human: formatBytesDetailed(state.memory_current_usage),
      },
      peak: {
        value: state.memory_peak_usage,
        hex: hex(state.memory_peak_usage),
        human: formatBytesDetailed(state.memory_peak_usage),
      },
      total_allocated: {
        value: state.memory_total_allocated,
        hex: hex(state.memory_total_allocated),
        human: formatBytesDetailed(state.memory_total_allocated),
      },
      total_freed: {
        value: state.memory_total_freed,
        hex: hex(state.memory_total_freed),
        human: formatBytesDetailed(state.memory_total_freed),
      },
      region_count: {
        active: regions.filter(r => !r.freed).length,
        total: regions.length,
      },
    },
    regions,
  };
}

function buildMemoryTextDump() {
  const dump = buildMemoryDumpData();
  const lines = [];
  lines.push('RINE MEMORY MAP DUMP');
  lines.push(`Generated: ${dump.generated_at}`);
  lines.push(`Status: ${dump.process_status}`);
  lines.push(`Exit Code: ${dump.process_exit_code == null ? 'n/a' : dump.process_exit_code}`);
  lines.push('');
  lines.push('Summary');
  lines.push(`Current: ${dump.memory_summary.current.human}`);
  lines.push(`Peak: ${dump.memory_summary.peak.human}`);
  lines.push(`Total Allocated: ${dump.memory_summary.total_allocated.human}`);
  lines.push(`Total Freed: ${dump.memory_summary.total_freed.human}`);
  lines.push(`Regions: ${dump.memory_summary.region_count.active} active / ${dump.memory_summary.region_count.total} total`);
  lines.push('');
  lines.push('Memory Regions');
  lines.push('Address | Size | Source | Status');
  lines.push('------- | ---- | ------ | ------');

  for (const region of dump.regions) {
    lines.push(`${region.address_hex} | ${formatBytesGrouped(region.size)} bytes (${region.size_human}) | ${region.source} | ${region.status}`);
  }

  return lines.join('\n');
}

function memoryDumpTimestamp() {
  return new Date().toISOString().replace(/[:]/g, '-').replace(/\..+$/, '');
}

async function saveMemoryDumpWithDialog(suggestedName, content) {
  try {
    const savedPath = await invoke('save_memory_dump', {
      suggestedName,
      content,
    });
    if (!savedPath) {
      return;
    }
    const note = document.getElementById('memory-final-note');
    if (note) {
      note.textContent = `Memory dump saved to ${savedPath}`;
    }
  } catch (err) {
    console.error('Failed to save memory dump:', err);
  }
}

async function exportMemoryJson() {
  const dump = buildMemoryDumpData();
  await saveMemoryDumpWithDialog(
    `rine-memory-map-${memoryDumpTimestamp()}.json`,
    JSON.stringify(dump, null, 2)
  );
}

async function exportMemoryText() {
  await saveMemoryDumpWithDialog(
    `rine-memory-map-${memoryDumpTimestamp()}.txt`,
    buildMemoryTextDump()
  );
}

function renderMemorySummary() {
  const el = document.getElementById('memory-summary');
  const activeRegions = state.memory_regions.filter(r => !r.freed).length;

  if (state.memory_regions.length === 0) {
    el.innerHTML = '<div class="placeholder">No memory activity yet…</div>';
    return;
  }

  el.innerHTML = [
    `<div class="memory-stat"><span class="memory-stat-label">Current Usage</span><span class="memory-stat-value">${formatBytesWithParens(state.memory_current_usage)}</span></div>`,
    `<div class="memory-stat"><span class="memory-stat-label">Peak Usage</span><span class="memory-stat-value">${formatBytesWithParens(state.memory_peak_usage)}</span></div>`,
    `<div class="memory-stat"><span class="memory-stat-label">Total Allocated</span><span class="memory-stat-value">${formatBytesWithParens(state.memory_total_allocated)}</span></div>`,
    `<div class="memory-stat"><span class="memory-stat-label">Total Freed</span><span class="memory-stat-value">${formatBytesWithParens(state.memory_total_freed)}</span></div>`,
    `<div class="memory-stat"><span class="memory-stat-label">Regions</span><span class="memory-stat-value">${activeRegions} active / ${state.memory_regions.length} total</span></div>`,
  ].join('');
}

function renderMemoryTable() {
  const tbody = document.getElementById('memory-tbody');
  const filterText = (document.getElementById('memory-filter').value || '').toLowerCase();
  const activeOnly = document.getElementById('memory-active-only').checked;

  const rows = state.memory_regions
    .slice()
    .sort((a, b) => Number(a.address) - Number(b.address))
    .filter(r => {
      if (activeOnly && r.freed) return false;
      if (!filterText) return true;
      return (
        hex(r.address).toLowerCase().includes(filterText)
        || String(r.size).toLowerCase().includes(filterText)
        || String(r.source || '').toLowerCase().includes(filterText)
      );
    });

  if (rows.length === 0) {
    tbody.innerHTML = '<tr><td colspan="4" class="placeholder">No matching memory regions</td></tr>';
    return;
  }

  tbody.innerHTML = rows.map(r =>
    `<tr class="${r.freed ? 'row-closed' : ''}">
      <td>${hex(r.address)}</td>
      <td>${formatBytesWithParens(r.size)}</td>
      <td>${esc(r.source)}</td>
      <td class="${r.freed ? 'status-closed' : 'status-open'}">${r.freed ? 'Freed' : 'Active'}</td>
    </tr>`
  ).join('');
}

function renderFinalMemoryMapping() {
  const wrap = document.getElementById('memory-final-wrap');
  const title = document.getElementById('memory-final-title');
  const note = document.getElementById('memory-final-note');

  if (state.exited == null) {
    wrap.hidden = true;
    return;
  }

  const crashed = state.exitCode == null;
  title.textContent = crashed ? 'Crash/Disconnect Memory Mapping' : 'Final Memory Mapping';
  if (!note.textContent || !note.textContent.startsWith('Memory dump saved to ')) {
    note.textContent = crashed
    ? 'Process disconnected unexpectedly. This snapshot shows the last known complete memory map.'
    : `Process exited with code ${state.exitCode}. Snapshot frozen at exit.`;
  }
  wrap.hidden = false;
}

function formatSnapshotRegionLabel(region, index) {
  const source = region.source || 'Unknown';
  return `${index + 1}. ${hex(region.address)} | ${formatBytesWithParens(region.size)} | ${source}`;
}

const SNAPSHOT_MAX_RENDER_BYTES = 2048;
const snapshotHexSelection = {
  mode: null,
  offset: null,
  lineStart: null,
  lineEnd: null,
};

function resetSnapshotHexSelection() {
  snapshotHexSelection.mode = null;
  snapshotHexSelection.offset = null;
  snapshotHexSelection.lineStart = null;
  snapshotHexSelection.lineEnd = null;
}

function escapeHtmlChar(ch) {
  if (ch === '&') return '&amp;';
  if (ch === '<') return '&lt;';
  if (ch === '>') return '&gt;';
  return ch;
}

function renderSnapshotGridHtml(bytes, startAddress) {
  const rows = [];
  const width = 16;
  for (let i = 0; i < bytes.length; i += width) {
    const chunk = bytes.slice(i, i + width);
    const lineStart = i;
    const lineEnd = i + chunk.length - 1;
    const address = hex(startAddress + i);
    const hexCells = [];
    const asciiCells = [];

    for (let j = 0; j < width; j += 1) {
      if (j < chunk.length) {
        const byte = chunk[j];
        const offset = i + j;
        const printable = byte >= 32 && byte <= 126 ? String.fromCharCode(byte) : '.';
        hexCells.push(`<span class="memory-byte-cell" data-byte-offset="${offset}">${byte.toString(16).padStart(2, '0')}</span>`);
        asciiCells.push(`<span class="memory-byte-cell" data-byte-offset="${offset}">${escapeHtmlChar(printable)}</span>`);
      } else {
        hexCells.push('<span class="memory-byte-cell memory-byte-empty">&nbsp;&nbsp;</span>');
        asciiCells.push('<span class="memory-byte-cell memory-byte-empty">&nbsp;</span>');
      }
    }

    rows.push(`
      <div class="memory-hex-row" data-line-start="${lineStart}" data-line-end="${lineEnd}">
        <div class="memory-addr-cell" data-line-start="${lineStart}" data-line-end="${lineEnd}">${address}</div>
        <div class="memory-hex-cells">${hexCells.join(' ')}</div>
        <div class="memory-ascii-cells">${asciiCells.join('')}</div>
        <div class="memory-reserved-space"></div>
      </div>
    `);
  }

  return `
    <div class="memory-hex-grid">
      <div class="memory-hex-header">
        <div>Address</div>
        <div>Hex Data</div>
        <div>ASCII</div>
        <div></div>
      </div>
      <div class="memory-hex-rows">
        ${rows.join('')}
      </div>
    </div>
  `;
}

function applySnapshotSelectionClasses() {
  const hexView = document.getElementById('memory-hex-view');
  if (!hexView) return;

  hexView.querySelectorAll('.memory-byte-selected, .memory-line-selected, .memory-addr-selected').forEach(el => {
    el.classList.remove('memory-byte-selected', 'memory-line-selected', 'memory-addr-selected');
  });

  if (snapshotHexSelection.mode === 'byte' && snapshotHexSelection.offset != null) {
    const offset = String(snapshotHexSelection.offset);
    hexView.querySelectorAll(`[data-byte-offset="${offset}"]`).forEach(el => {
      el.classList.add('memory-byte-selected');
    });
    return;
  }

  if (snapshotHexSelection.mode === 'line' && snapshotHexSelection.lineStart != null && snapshotHexSelection.lineEnd != null) {
    const start = snapshotHexSelection.lineStart;
    const end = snapshotHexSelection.lineEnd;
    hexView.querySelectorAll('[data-byte-offset]').forEach(el => {
      const value = Number(el.dataset.byteOffset);
      if (value >= start && value <= end) {
        el.classList.add('memory-line-selected');
      }
    });

    hexView.querySelectorAll('.memory-addr-cell').forEach(el => {
      const rowStart = Number(el.dataset.lineStart);
      const rowEnd = Number(el.dataset.lineEnd);
      if (rowStart === start && rowEnd === end) {
        el.classList.add('memory-addr-selected');
      }
    });
  }
}

function handleSnapshotHexClick(event) {
  const rawTarget = event.target;
  const target = rawTarget instanceof HTMLElement
    ? rawTarget
    : (rawTarget && rawTarget.parentElement instanceof HTMLElement ? rawTarget.parentElement : null);
  if (!target) return;

  const addr = target.closest('.memory-addr-cell');
  if (addr instanceof HTMLElement) {
    snapshotHexSelection.mode = 'line';
    snapshotHexSelection.offset = null;
    snapshotHexSelection.lineStart = Number(addr.dataset.lineStart);
    snapshotHexSelection.lineEnd = Number(addr.dataset.lineEnd);
    applySnapshotSelectionClasses();
    return;
  }

  const byte = target.closest('.memory-byte-cell[data-byte-offset]');
  if (byte instanceof HTMLElement) {
    snapshotHexSelection.mode = 'byte';
    snapshotHexSelection.offset = Number(byte.dataset.byteOffset);
    snapshotHexSelection.lineStart = null;
    snapshotHexSelection.lineEnd = null;
    applySnapshotSelectionClasses();
  }
}

async function loadSnapshotMetaAndRegions() {
  if (!state.memory_snapshot || !state.memory_snapshot.json_path) {
    return;
  }

  try {
    const meta = await invoke('load_memory_snapshot_meta', {
      jsonPath: state.memory_snapshot.json_path,
    });
    state.memory_snapshot_meta = meta;

    const viewer = document.getElementById('memory-snapshot-viewer');
    const regionSelect = document.getElementById('memory-snapshot-region');
    const hexView = document.getElementById('memory-hex-view');
    const regions = Array.isArray(meta.regions) ? meta.regions : [];

    regionSelect.innerHTML = regions.map((r, i) =>
      `<option value="${i}">${esc(formatSnapshotRegionLabel(r, i))}</option>`
    ).join('');

    if (regions.length > 0) {
      viewer.hidden = false;
      regionSelect.value = '0';
      loadSnapshotHexChunk();
    } else {
      viewer.hidden = true;
      hexView.textContent = 'No snapshot regions available.';
    }
  } catch (err) {
    console.error('Failed to load snapshot metadata:', err);
  }
}

async function loadSnapshotHexChunk() {
  const hexView = document.getElementById('memory-hex-view');
  const regionSelect = document.getElementById('memory-snapshot-region');

  if (!state.memory_snapshot_meta || !Array.isArray(state.memory_snapshot_meta.regions)) {
    hexView.textContent = 'No snapshot metadata loaded.';
    return;
  }

  const idx = Number(regionSelect.value || 0);
  const region = state.memory_snapshot_meta.regions[idx];
  if (!region) {
    hexView.innerHTML = '<div class="memory-hex-empty">No region selected.</div>';
    resetSnapshotHexSelection();
    return;
  }

  const totalLength = Math.max(0, Number(region.size || 0));
  if (totalLength <= 0) {
    hexView.innerHTML = '<div class="memory-hex-empty">Region is empty.</div>';
    resetSnapshotHexSelection();
    return;
  }

  const displayLength = Math.min(totalLength, SNAPSHOT_MAX_RENDER_BYTES);
  const truncated = totalLength > displayLength;
  resetSnapshotHexSelection();
  hexView.innerHTML = '<div class="memory-hex-empty">Loading snapshot preview…</div>';

  try {
    const chunk = await invoke('read_memory_snapshot_chunk', {
      binPath: state.memory_snapshot.bin_path,
      offset: Number(region.file_offset || 0),
      length: displayLength,
    });

    const metaCard = `
      <div class="memory-info-card">
        <div class="memory-info-item">
          <span class="memory-info-label">Region:</span>
          <span class="memory-info-value">${esc(hex(region.address))}</span>
        </div>
        <div class="memory-info-item">
          <span class="memory-info-label">Size:</span>
          <span class="memory-info-value">${esc(formatBytesWithParens(region.size))}</span>
        </div>
        <div class="memory-info-item">
          <span class="memory-info-label">Source:</span>
          <span class="memory-info-value">${esc(region.source || 'Unknown')}</span>
        </div>
        ${truncated ? `<div class="memory-info-note">Previewing ${esc(formatBytesGrouped(displayLength))} of ${esc(formatBytesGrouped(totalLength))} bytes to keep the viewer responsive.</div>` : ''}
      </div>
    `;
    const viewer = document.getElementById('memory-snapshot-viewer');
    const toolbar = viewer.querySelector('.memory-snapshot-toolbar');
    const metaPlaceholder = viewer.querySelector('.memory-info-card') || null;
    if (metaPlaceholder) {
      metaPlaceholder.remove();
    }
    if (toolbar) {
      toolbar.insertAdjacentHTML('afterend', metaCard);
    }

    hexView.innerHTML = renderSnapshotGridHtml(chunk, Number(region.address || 0));
  } catch (err) {
    console.error('Failed to read snapshot chunk:', err);
    hexView.innerHTML = `<div class="memory-hex-empty">Failed to read snapshot chunk: ${esc(String(err))}</div>`;
  }
}

function handleMemoryEvent(event) {
  switch (event.type) {
    case 'MemoryAllocated':
      state.memory_regions.push({
        address: event.address,
        size: event.size,
        source: event.source,
        freed: false,
      });
      state.memory_total_allocated += event.size;
      state.memory_current_usage += event.size;
      state.memory_peak_usage = Math.max(state.memory_peak_usage, state.memory_current_usage);
      renderMemorySummary();
      renderMemoryTable();
      return true;
    case 'MemoryFreed': {
      let freedSize = event.size;
      for (let i = state.memory_regions.length - 1; i >= 0; i--) {
        const region = state.memory_regions[i];
        if (!region.freed && region.address === event.address) {
          region.freed = true;
          freedSize = region.size;
          break;
        }
      }
      state.memory_total_freed += freedSize;
      state.memory_current_usage = Math.max(0, state.memory_current_usage - freedSize);
      renderMemorySummary();
      renderMemoryTable();
      return true;
    }
    case 'MemorySnapshotReady':
      state.memory_snapshot = {
        json_path: event.json_path,
        bin_path: event.bin_path,
        region_count: event.region_count,
        total_bytes: event.total_bytes,
      };
      renderFinalMemoryMapping();
      const note = document.getElementById('memory-final-note');
      if (note) {
        note.textContent = `Snapshot captured: ${event.region_count} regions, ${formatBytesWithParens(event.total_bytes)}.`;
      }
      loadSnapshotMetaAndRegions();
      return true;
    default:
      return false;
  }
}

function hydrateMemoryState(snap) {
  if (snap.memory_regions && snap.memory_regions.length) {
    state.memory_regions = snap.memory_regions;
  }
  if (typeof snap.memory_current_usage === 'number') {
    state.memory_current_usage = snap.memory_current_usage;
  }
  if (typeof snap.memory_peak_usage === 'number') {
    state.memory_peak_usage = snap.memory_peak_usage;
  }
  if (typeof snap.memory_total_allocated === 'number') {
    state.memory_total_allocated = snap.memory_total_allocated;
  }
  if (typeof snap.memory_total_freed === 'number') {
    state.memory_total_freed = snap.memory_total_freed;
  }
  if (snap.memory_snapshot) {
    state.memory_snapshot = snap.memory_snapshot;
    loadSnapshotMetaAndRegions();
  }
}

function bindMemoryUi() {
  document.querySelectorAll('.memory-subtab').forEach(btn => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('.memory-subtab').forEach(b => b.classList.remove('active'));
      document.querySelectorAll('.memory-panel').forEach(panel => panel.classList.remove('active'));
      btn.classList.add('active');
      document.getElementById(`memory-panel-${btn.dataset.memoryTab}`).classList.add('active');
    });
  });

  document.getElementById('memory-filter').addEventListener('input', renderMemoryTable);
  document.getElementById('memory-active-only').addEventListener('change', renderMemoryTable);
  document.getElementById('memory-export-json').addEventListener('click', exportMemoryJson);
  document.getElementById('memory-export-text').addEventListener('click', exportMemoryText);
  document.getElementById('memory-snapshot-region').addEventListener('change', () => {
    loadSnapshotHexChunk();
  });
  document.getElementById('memory-hex-view').addEventListener('click', handleSnapshotHexClick);
}
