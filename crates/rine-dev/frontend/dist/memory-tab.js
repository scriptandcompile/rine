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
  note.textContent = crashed
    ? 'Process disconnected unexpectedly. This snapshot shows the last known complete memory map.'
    : `Process exited with code ${state.exitCode}. Snapshot frozen at exit.`;
  wrap.hidden = false;
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
}

function bindMemoryUi() {
  document.getElementById('memory-filter').addEventListener('input', renderMemoryTable);
  document.getElementById('memory-active-only').addEventListener('change', renderMemoryTable);
  document.getElementById('memory-export-json').addEventListener('click', exportMemoryJson);
  document.getElementById('memory-export-text').addEventListener('click', exportMemoryText);
}
