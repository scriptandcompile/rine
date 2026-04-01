// ANSI escape code → HTML converter

function escAnsiHtml(s) {
  const d = document.createElement("div");
  d.textContent = s;
  return d.innerHTML;
}

function ansiToHtml(raw) {
  const COLORS = [
    "#45475a", "#f38ba8", "#a6e3a1", "#f9e2af",
    "#89b4fa", "#cba6f7", "#94e2d5", "#bac2de",
  ];
  const BRIGHT = [
    "#585b70", "#f37799", "#94dba0", "#f2d5a0",
    "#74c7ec", "#b4a0e5", "#80dbc8", "#cdd6f4",
  ];

  let fg = null, bg = null, bold = false, dim = false, italic = false, underline = false;
  let out = "";

  const parts = raw.split(/(\x1b\[[0-9;]*m)/);
  for (const part of parts) {
    const m = part.match(/^\x1b\[([0-9;]*)m$/);
    if (!m) {
      if (!part) continue;
      const styles = [];
      if (fg) styles.push("color:" + fg);
      if (bg) styles.push("background:" + bg);
      if (bold) styles.push("font-weight:bold");
      if (dim) styles.push("opacity:0.6");
      if (italic) styles.push("font-style:italic");
      if (underline) styles.push("text-decoration:underline");
      if (styles.length) {
        out += '<span style="' + styles.join(";") + '">' + escAnsiHtml(part) + "</span>";
      } else {
        out += escAnsiHtml(part);
      }
      continue;
    }
    const codes = m[1] ? m[1].split(";").map(Number) : [0];
    for (let i = 0; i < codes.length; i++) {
      const c = codes[i];
      if (c === 0) { fg = bg = null; bold = dim = italic = underline = false; }
      else if (c === 1) bold = true;
      else if (c === 2) dim = true;
      else if (c === 3) italic = true;
      else if (c === 4) underline = true;
      else if (c === 22) { bold = false; dim = false; }
      else if (c === 23) italic = false;
      else if (c === 24) underline = false;
      else if (c >= 30 && c <= 37) fg = bold ? BRIGHT[c - 30] : COLORS[c - 30];
      else if (c === 38 && codes[i + 1] === 5) { fg = ansi256(codes[i + 2] || 0); i += 2; }
      else if (c === 39) fg = null;
      else if (c >= 40 && c <= 47) bg = COLORS[c - 40];
      else if (c === 48 && codes[i + 1] === 5) { bg = ansi256(codes[i + 2] || 0); i += 2; }
      else if (c === 49) bg = null;
      else if (c >= 90 && c <= 97) fg = BRIGHT[c - 90];
      else if (c >= 100 && c <= 107) bg = BRIGHT[c - 100];
    }
  }
  return out;
}

function ansi256(n) {
  if (n < 8) return [
    "#45475a", "#f38ba8", "#a6e3a1", "#f9e2af",
    "#89b4fa", "#cba6f7", "#94e2d5", "#bac2de",
  ][n];
  if (n < 16) return [
    "#585b70", "#f37799", "#94dba0", "#f2d5a0",
    "#74c7ec", "#b4a0e5", "#80dbc8", "#cdd6f4",
  ][n - 8];
  if (n < 232) {
    const i = n - 16;
    const r = Math.floor(i / 36), g = Math.floor((i % 36) / 6), b = i % 6;
    const v = c => c ? 55 + c * 40 : 0;
    return "#" + [r, g, b].map(c => v(c).toString(16).padStart(2, "0")).join("");
  }
  const g = 8 + (n - 232) * 10;
  return "#" + g.toString(16).padStart(2, "0").repeat(3);
}
