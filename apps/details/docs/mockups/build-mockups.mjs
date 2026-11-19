// Generates the Details mockups as static HTML. No dependencies.
//   node build-mockups.mjs              writes index.html next to this file
//   node build-mockups.mjs <canvasDir>  also writes <canvasDir>/project/*.dc.html
// Every number is sample data, chosen so totals add up across screens.
import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const DARK = {
  bg: '#121418', card: '#1A1D22', raised: '#252930', line: '#2E333B',
  ink: '#F0F2F5', ink2: '#AEB5BF', ink3: '#8F97A2', link: '#6DAEFF',
  cpu: '#4C9BFF', memory: '#A98BFF', disk: '#E8A33D', network: '#3FC48A', gpu: '#F06A9B', neural: '#3CC9D6',
  battery: '#9CCF4A', sound: '#E66FD8', bluetooth: '#7F97FF', projects: '#FF8A4C', agents: '#B48CFF', containers: '#5FB8E8',
  strain: '#FF6B6B', good: '#3DBF7A',
  seg: ['#3D83E6', '#D9622B', '#1FA878', '#A56CE6', '#6B7480'], track: '#2E333B',
};
const LIGHT = {
  bg: '#E9ECF0', card: '#FFFFFF', raised: '#EEF0F3', line: '#D8DCE2',
  ink: '#15181C', ink2: '#4A515B', ink3: '#5F6772', link: '#1560C9',
  cpu: '#1F6FE0', memory: '#7A52E8', disk: '#B8730F', network: '#128A5A', gpu: '#D13A72', neural: '#0E8C9C',
  battery: '#5F8F12', sound: '#B83FA8', bluetooth: '#4560D8', projects: '#D65C1C', agents: '#8450D8', containers: '#1B83B8',
  strain: '#D42F2F', good: '#15894F',
  seg: ['#2A6FD6', '#D0561F', '#14936A', '#9A63DC', '#8A929C'], track: '#E1E4E9',
};
let t = DARK;

const FONT = `-apple-system, BlinkMacSystemFont, 'SF Pro Text', 'Helvetica Neue', Arial, sans-serif`;
const MONO = `ui-monospace, 'SF Mono', Menlo, Consolas, monospace`;

const ICONS = {
  overview: 'M4 4h7v7H4zM13 4h7v7h-7zM4 13h7v7H4zM13 13h7v7h-7z',
  cpu: 'M7 7h10v10H7zM10 3v4M14 3v4M10 17v4M14 17v4M3 10h4M3 14h4M17 10h4M17 14h4',
  memory: 'M3 8h18v8H3zM7 8v8M11 8v8M15 8v8M6 16v3M18 16v3',
  disk: 'M3 13l3-8h12l3 8v6H3zM3 13h18M16.5 16h.01',
  network: 'M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18zM3 12h18M12 3c3 3 3 15 0 18M12 3c-3 3-3 15 0 18',
  gpu: 'M12 3l9 5l-9 5l-9-5zM3 12l9 5l9-5M3 16l9 5l9-5',
  neural: 'M6 6h.01M18 6h.01M12 12h.01M6 18h.01M18 18h.01M6 6l6 6l6-6M6 18l6-6l6 6',
  battery: 'M3 8h15v8H3zM21 11v2M6 11v2M9 11v2',
  sound: 'M4 10v4h3l5 4V6L7 10zM16 9a4 4 0 0 1 0 6M18.5 6.5a8 8 0 0 1 0 11',
  bluetooth: 'M7 7l10 10l-5 4V3l5 4L7 17',
  projects: 'M3 6h6l2 2h10v11H3z',
  agents: 'M5 9h14v10H5zM12 4v5M9 14h.01M15 14h.01M2 13v3M22 13v3',
  containers: 'M3 8l9-5l9 5v8l-9 5l-9-5zM3 8l9 5l9-5M12 13v8',
  chevron: 'M9 6l6 6l-6 6', down: 'M6 9l6 6l6-6', x: 'M6 6l12 12M18 6L6 18',
  gear: 'M12 8.5a3.5 3.5 0 1 0 0 7a3.5 3.5 0 1 0 0-7zM12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2',
  power: 'M12 3v9M6.5 7a8 8 0 1 0 11 0', search: 'M11 4a7 7 0 1 0 0 14a7 7 0 1 0 0-14zM16 16l5 5',
  columns: 'M4 4h16v16H4zM10 4v16M15 4v16', share: 'M12 15V4M8 8l4-4l4 4M5 13v7h14v-7',
  info: 'M12 3a9 9 0 1 0 0 18a9 9 0 1 0 0-18zM12 11v5M12 8h.01', check: 'M5 12l5 5l9-10',
  bolt: 'M13 3L5 14h6l-1 7l8-11h-6z', mic: 'M9 4h6v10H9zM5 11a7 7 0 0 0 14 0M12 18v3', therm: 'M10 4h4v10a4 4 0 1 1-4 0z',
  folder: 'M3 6h6l2 2h10v11H3z', alert: 'M12 4l9 16H3zM12 10v4M12 17h.01', lock: 'M6 11h12v9H6zM8 11V8a4 4 0 0 1 8 0v3',
};
const TABS = [['overview', 'Overview'], ['cpu', 'CPU'], ['memory', 'Memory'], ['disk', 'Disk'], ['network', 'Network'], ['gpu', 'GPU'], ['neural', 'Neural'],
  ['battery', 'Battery'], ['sound', 'Sound'], ['bluetooth', 'Bluetooth'], ['projects', 'Projects'], ['agents', 'Agents'], ['containers', 'Containers']];

const icon = (n, size = 14, color) => `<svg aria-hidden="true" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="${color ?? t.ink2}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" style="flex: none"><path d="${ICONS[n]}"></path></svg>`;
const chip = (n, c, size = 26) => `<span style="display: inline-flex; align-items: center; justify-content: center; width: ${size}px; height: ${size}px; border-radius: 7px; background: ${c}2B; flex: none">${icon(n, Math.round(size * 0.58), c)}</span>`;
const mark = (s, size = 24) => `<span style="display: inline-flex; align-items: center; justify-content: center; width: ${size}px; height: ${size}px; border-radius: ${Math.round(size * 0.26)}px; background: ${t.raised}; border: 1px solid ${t.line}; color: ${t.ink}; font-size: ${Math.round(size * 0.42)}px; font-weight: 600; flex: none">${s}</span>`;
const card = (body, extra = '') => `<div style="background: ${t.card}; border: 1px solid ${t.line}; border-radius: 12px; padding: 16px; min-width: 0; box-sizing: border-box; ${extra}">${body}</div>`;
const small = (s, c) => `<span style="font-size: 12px; color: ${c ?? t.ink3}">${s}</span>`;
const stat = (l, v) => `<div style="display: flex; flex-direction: column; gap: 2px; min-width: 0">${small(l)}<span style="font-size: 13px; font-weight: 600; color: ${t.ink}">${v}</span></div>`;
const big = (v, u, size = 34) => `<div style="display: flex; align-items: baseline; gap: 5px"><span style="font-size: ${size}px; font-weight: 600; letter-spacing: -0.5px; color: ${t.ink}; line-height: 1.1">${v}</span><span style="font-size: ${Math.round(size * 0.48)}px; color: ${t.ink2}">${u}</span></div>`;
const pill = (txt, kind = 'plain') => {
  const c = { good: t.good, red: t.strain, plain: t.ink3, wait: t.disk, warn: t.projects }[kind];
  const dot = kind === 'plain' ? '' : `<span style="width: 6px; height: 6px; border-radius: 3px; background: ${c}; flex: none"></span>`;
  return `<span style="display: inline-flex; align-items: center; gap: 5px; padding: 2px 8px; border-radius: 10px; background: ${kind === 'plain' ? t.raised : c + '26'}; color: ${t.ink}; font-size: 11.5px; font-weight: 500; white-space: nowrap">${dot}${txt}</span>`;
};
const portChip = (p) => `<span style="padding: 2px 7px; border-radius: 6px; border: 1px solid ${t.projects}80; color: ${t.ink}; font-family: ${MONO}; font-size: 11.5px">:${p}</span>`;
const btn = (label, { kind = 'plain', ic, extra = '' } = {}) => {
  const bg = kind === 'danger' ? t.strain : kind === 'primary' ? t.cpu : t.raised;
  const fg = kind === 'plain' ? t.ink : '#0E1013';
  return `<button type="button" style="display: inline-flex; align-items: center; justify-content: center; gap: 6px; min-height: 28px; padding: 5px 12px; border-radius: 8px; border: 1px solid ${kind === 'plain' ? t.line : bg}; background: ${bg}; color: ${fg}; font: inherit; font-size: 12.5px; font-weight: 500; cursor: pointer; ${extra}">${ic ? icon(ic, 13, fg) : ''}${label}</button>`;
};
const iconBtn = (n, label) => `<button type="button" aria-label="${label}" style="display: inline-flex; align-items: center; justify-content: center; width: 28px; height: 28px; border-radius: 8px; border: 1px solid ${t.line}; background: ${t.raised}; cursor: pointer; flex: none; padding: 0">${icon(n, 14, t.ink2)}</button>`;
const meter = (pct, c, h = 4) => `<div style="height: ${h}px; border-radius: ${h / 2}px; background: ${t.track}; overflow: hidden"><div style="width: ${pct}%; height: 100%; border-radius: ${h / 2}px; background: ${c}"></div></div>`;
const series = (n, seed, base, amp) => Array.from({ length: n }, (_, i) => Math.max(1, Math.min(100, base + amp * Math.sin(i * 0.55 + seed) + (amp / 2) * Math.sin(i * 1.7 + seed * 2))));

const strip = (data, c, h = 34, label = '') => {
  const W = 300, gap = 2, bw = (W - gap * (data.length - 1)) / data.length;
  const rects = data.map((v, i) => { const bh = Math.max(1.5, (v / 100) * h); return `<rect x="${(i * (bw + gap)).toFixed(2)}" y="${(h - bh).toFixed(2)}" width="${bw.toFixed(2)}" height="${bh.toFixed(2)}" rx="1" fill="${c}"></rect>`; }).join('');
  return `<svg role="img" aria-label="${label}" viewBox="0 0 ${W} ${h}" preserveAspectRatio="none" style="display: block; width: 100%; height: ${h}px">${rects}</svg>`;
};
const spark = (data, c, w = 96, h = 26) => {
  const pts = data.map((v, i) => [(i / (data.length - 1)) * (w - 4), h - 2 - (v / 100) * (h - 4)]);
  const line = pts.map((p) => `${p[0].toFixed(1)},${p[1].toFixed(1)}`).join(' ');
  const end = pts[pts.length - 1];
  return `<svg aria-hidden="true" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}" style="flex: none"><polygon points="0,${h} ${line} ${end[0].toFixed(1)},${h}" fill="${c}" fill-opacity="0.14"></polygon><polyline points="${line}" fill="none" stroke="${c}" stroke-width="1.5" stroke-linejoin="round"></polyline></svg>`;
};
// Live area chart on a fixed percent scale: faint grid, 10% wash, 2px line, emphasised end point.
const area = (data, c, { w = 760, h = 190, ticks = [0, 25, 50, 75, 100], xs = [], unit = '%', label = '' } = {}) => {
  const L = 34, R = 10, T = 8, B = 22, pw = w - L - R, ph = h - T - B, y = (v) => T + ph - (v / 100) * ph;
  const pts = data.map((v, i) => [L + (i / (data.length - 1)) * pw, y(v)]);
  const line = pts.map((p) => `${p[0].toFixed(1)},${p[1].toFixed(1)}`).join(' ');
  const e = pts[pts.length - 1];
  const grid = ticks.map((k) => `<line x1="${L}" x2="${w - R}" y1="${y(k)}" y2="${y(k)}" stroke="${t.line}" stroke-width="1"></line><text x="${L - 6}" y="${y(k) + 4}" font-size="11" text-anchor="end" fill="${t.ink3}">${k}${unit}</text>`).join('');
  const xl = xs.map((s, i) => `<text x="${L + (i / (xs.length - 1)) * pw}" y="${h - 5}" font-size="11" text-anchor="${i === 0 ? 'start' : i === xs.length - 1 ? 'end' : 'middle'}" fill="${t.ink3}">${s}</text>`).join('');
  return `<svg role="img" aria-label="${label}" viewBox="0 0 ${w} ${h}" style="display: block; width: 100%; height: auto">${grid}${xl}<polygon points="${L},${T + ph} ${line} ${e[0].toFixed(1)},${T + ph}" fill="${c}" fill-opacity="0.12"></polygon><polyline points="${line}" fill="none" stroke="${c}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"></polyline><circle cx="${e[0].toFixed(1)}" cy="${e[1].toFixed(1)}" r="4" fill="${c}" stroke="${t.card}" stroke-width="2"></circle></svg>`;
};
const columns = (data, c, { w = 1180, h = 210, ticks = [0, 25, 50, 75, 100], xs = [], label = '' } = {}) => {
  const L = 38, R = 6, T = 8, B = 24, pw = w - L - R, ph = h - T - B, y = (v) => T + ph - (v / 100) * ph, slot = pw / data.length, bw = Math.min(24, slot - 2);
  const grid = ticks.map((k) => `<line x1="${L}" x2="${w - R}" y1="${y(k)}" y2="${y(k)}" stroke="${t.line}" stroke-width="1"></line><text x="${L - 6}" y="${y(k) + 4}" font-size="11" text-anchor="end" fill="${t.ink3}">${k}%</text>`).join('');
  const bars = data.map((v, i) => { const x = L + i * slot + (slot - bw) / 2, top = y(v), r = 3, bot = T + ph; return `<path d="M${x.toFixed(1)} ${bot}V${(top + r).toFixed(1)}Q${x.toFixed(1)} ${top.toFixed(1)} ${(x + r).toFixed(1)} ${top.toFixed(1)}H${(x + bw - r).toFixed(1)}Q${(x + bw).toFixed(1)} ${top.toFixed(1)} ${(x + bw).toFixed(1)} ${(top + r).toFixed(1)}V${bot}Z" fill="${c}"></path>`; }).join('');
  const xl = xs.map(([i, s]) => `<text x="${L + i * slot + slot / 2}" y="${h - 6}" font-size="11" text-anchor="middle" fill="${t.ink3}">${s}</text>`).join('');
  return `<svg role="img" aria-label="${label}" viewBox="0 0 ${w} ${h}" style="display: block; width: 100%; height: auto">${grid}${bars}${xl}</svg>`;
};
const ring = (pct, c, size = 58) => {
  const r = (size - 7) / 2, len = 2 * Math.PI * r;
  return `<svg role="img" aria-label="${pct}% used" width="${size}" height="${size}" viewBox="0 0 ${size} ${size}" style="flex: none"><circle cx="${size / 2}" cy="${size / 2}" r="${r}" fill="none" stroke="${t.track}" stroke-width="6"></circle><circle cx="${size / 2}" cy="${size / 2}" r="${r}" fill="none" stroke="${c}" stroke-width="6" stroke-linecap="round" stroke-dasharray="${(len * pct / 100).toFixed(1)} ${len.toFixed(1)}" transform="rotate(-90 ${size / 2} ${size / 2})"></circle><text x="${size / 2}" y="${size / 2 + 4}" text-anchor="middle" font-size="13" font-weight="600" fill="${t.ink}">${pct}%</text></svg>`;
};
// Part to whole, segmented bar with a legend and every value in text.
const segmented = (parts) => {
  const total = parts.reduce((a, p) => a + p[1], 0);
  const bar = parts.map((p, i) => `<div style="flex: ${p[1]} 1 0; height: 100%; background: ${t.seg[i]}; border-radius: 3px"></div>`).join('');
  const rows = parts.map((p, i) => `<div style="display: flex; align-items: center; gap: 8px; font-size: 12.5px"><span style="width: 8px; height: 8px; border-radius: 2px; background: ${t.seg[i]}; flex: none"></span><span style="flex: 1; color: ${t.ink2}">${p[0]}</span><span style="color: ${t.ink}; font-weight: 500">${p[2]}</span></div>`).join('');
  return `<div style="display: flex; flex-direction: column; gap: 12px"><div role="img" aria-label="${parts.map((p) => p[0] + ' ' + p[2]).join(', ')}" style="display: flex; gap: 2px; height: 16px">${bar}</div><div style="display: flex; flex-direction: column; gap: 7px">${rows}</div></div>`;
};
const tabBar = (active) => TABS.map(([id, label]) => {
  const on = id === active, c = t[id] ?? t.cpu;
  return `<button type="button" ${on ? 'aria-current="page"' : ''} style="display: inline-flex; align-items: center; gap: 5px; padding: 5px 8px; border-radius: 14px; border: 0; background: ${on ? c + '2E' : 'transparent'}; color: ${on ? t.ink : t.ink2}; font: inherit; font-size: 12px; font-weight: ${on ? 600 : 400}; cursor: pointer; white-space: nowrap">${icon(id, 13, on ? c : t.ink3)}${label}</button>`;
}).join('');
const lights = `<div aria-hidden="true" style="display: flex; gap: 7px; flex: none"><span style="width: 12px; height: 12px; border-radius: 6px; background: #FF5F57"></span><span style="width: 12px; height: 12px; border-radius: 6px; background: #FEBC2E"></span><span style="width: 12px; height: 12px; border-radius: 6px; background: #28C840"></span></div>`;
const win = (active, body) => `<div style="min-height: 100%; background: ${t.bg}; color: ${t.ink}; font-family: ${FONT}; font-variant-numeric: tabular-nums; font-size: 13px; box-sizing: border-box">
<div style="display: flex; flex-wrap: wrap; align-items: center; gap: 10px 14px; padding: 10px 14px; border-bottom: 1px solid ${t.line}">${lights}<nav aria-label="Tabs" style="display: flex; flex-wrap: wrap; gap: 1px; justify-content: center; flex: 1; min-width: 0">${tabBar(active)}</nav></div>
<div style="display: flex; flex-direction: column; gap: 14px; padding: 16px">${body}</div></div>`;
const grid3 = (body) => `<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 14px">${body}</div>`;
const section = (title, right = '') => `<div style="display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: 8px"><h2 style="margin: 0; font-size: 13px; font-weight: 600; color: ${t.ink}">${title}</h2>${right}</div>`;
const kv = (k, v) => `<div style="display: grid; grid-template-columns: 112px minmax(0, 1fr); gap: 12px; font-size: 12.5px; line-height: 1.45"><span style="text-align: right; color: ${t.ink3}">${k}</span><span style="color: ${t.ink}; min-width: 0; overflow-wrap: anywhere">${v}</span></div>`;
const code = (s) => `<span style="font-family: ${MONO}; font-size: 11.5px">${s}</span>`;
const link = (s) => `<a href="#" style="color: ${t.link}; text-decoration: none">${s}</a>`;

/* ---------- Overview ---------- */
const subCard = (id, label, cap, value, unit, stats, data, status = '') => card(`
<div style="display: flex; flex-direction: column; gap: 12px">
<div style="display: flex; align-items: center; gap: 8px">${chip(id, t[id])}<span style="font-weight: 600; flex: 1">${label}</span>${icon('chevron', 13, t.ink3)}</div>
<div style="display: flex; flex-direction: column; gap: 3px">${small(cap)}<div style="display: flex; align-items: center; justify-content: space-between; gap: 8px">${big(value, unit)}${status}</div></div>
<div style="display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px">${stats.map((s) => stat(s[0], s[1])).join('')}</div>
${strip(data, t[id], 34, label + ' history, last 15 minutes')}
</div>`);
const overview = () => win('overview', `
${grid3([
    subCard('cpu', 'CPU', 'Now', '38', '%', [['User', '26%'], ['System', '12%'], ['Average today', '41%']], series(44, 1, 40, 10)),
    subCard('memory', 'Memory', 'In use of 48 GB', '35.7', 'GB', [['App', '22.4 GB'], ['Wired', '4.2 GB'], ['Compressed', '9.1 GB']], series(44, 2, 74, 2), pill('Normal', 'good')),
    subCard('gpu', 'GPU', 'Apple M4 Pro', '46', '%', [['Memory', '1.69 GB'], ['Average', '47%'], ['Peak', '75%']], series(44, 3, 42, 13)),
    subCard('disk', 'Disk', 'Free of 994.7 GB', '412.3', 'GB', [['Reading', '5.5 MB/s'], ['Writing', '6.8 MB/s'], ['Written today', '39.2 GB']], series(44, 4, 24, 20)),
    subCard('network', 'Network', 'Downloading', '5.3', 'MB/s', [['Uploading', '240 kB/s'], ['Today', '4.7 GB'], ['Last 7 days', '48.3 GB']], series(44, 5, 28, 22)),
    subCard('battery', 'Battery', 'On battery', '63', '%', [['Remaining', '2 h 24 min'], ['Power draw', '18.2 W'], ['Health', '91%']], series(44, 6, 68, 3)),
  ].join(''))}
${grid3([
    card(`<div style="display: flex; flex-direction: column; gap: 12px">${section('Memory by type', small('48 GB'))}${segmented([['App', 22.4, '22.4 GB'], ['Wired', 4.2, '4.2 GB'], ['Compressed', 9.1, '9.1 GB'], ['Cached', 8.3, '8.3 GB'], ['Free', 4.0, '4.0 GB']])}</div>`),
    card(`<div style="display: flex; flex-direction: column; gap: 12px">${section('Memory by app', small('22.4 GB'))}${segmented([['Google Chrome', 8.6, '8.6 GB'], ['VS Code', 4.1, '4.1 GB'], ['Docker', 3.4, '3.4 GB'], ['Terminal', 2.8, '2.8 GB'], ['Other', 3.5, '3.5 GB']])}</div>`),
    card(`<div style="display: flex; flex-direction: column; gap: 12px">${section('Power by app', small('11.0 W of 18.2 W'))}${segmented([['Google Chrome', 4.1, '4.1 W'], ['Docker', 2.3, '2.3 W'], ['VS Code', 1.6, '1.6 W'], ['Terminal', 1.2, '1.2 W'], ['Other', 1.8, '1.8 W']])}</div>`),
  ].join(''))}
<div style="display: flex; flex-wrap: wrap; gap: 8px; align-items: center">${btn('Over Time')}${btn('Peaks')}${btn('Worth a Look · 3')}<span style="flex: 1"></span>${small('Up 6 days 4 hours · measured 2 seconds ago')}</div>`);

/* ---------- CPU ---------- */
const appRow = (m, name, sub, val, pct, c, { top = false, open = false, actions = '' } = {}) => `<div style="display: grid; grid-template-columns: 16px 28px minmax(140px, 1fr) minmax(90px, 240px) 66px 64px; align-items: center; gap: 10px; padding: 7px 10px; border-radius: 8px; background: ${top ? t.raised : 'transparent'}">
${icon(open ? 'down' : 'chevron', 12, t.ink3)}${mark(m, 26)}<div style="display: flex; flex-direction: column; min-width: 0"><span style="font-weight: 500">${name}</span>${small(sub)}</div><div>${meter(pct, c)}</div><span style="text-align: right; font-weight: 600">${val}</span><span style="display: flex; justify-content: flex-end">${actions}</span></div>`;
const procRow = (name, pid, cpu, mem, tag = '') => `<div style="display: grid; grid-template-columns: minmax(180px, 1fr) 70px 70px; gap: 10px; padding: 5px 10px 5px 64px; font-size: 12.5px; align-items: center"><span style="display: flex; gap: 8px; align-items: center; flex-wrap: wrap">${name} ${small(pid)}${tag}</span><span style="text-align: right; color: ${t.ink2}">${cpu}</span><span style="text-align: right; color: ${t.ink2}">${mem}</span></div>`;
const coreChip = (s) => `<span style="padding: 2px 7px; border-radius: 6px; background: ${t.raised}; font-size: 12px; font-weight: 600">${s}</span>`;
const cpuTab = () => win('cpu', `
${card(`<div style="display: flex; flex-wrap: wrap; gap: 20px; align-items: stretch"><div style="flex: 1 1 180px; display: flex; flex-direction: column; gap: 14px">${small('Now, of this whole Mac')}${big('38', '%', 44)}<div style="display: flex; gap: 22px">${stat('Average today', '41%')}${stat('Load', '4.96')}</div>${small('100% means every core is busy. Switch to per core in Settings.')}</div><div style="flex: 999 1 520px; min-width: 0">${area(series(60, 1, 40, 12).concat([38]), t.cpu, { xs: ['15 min ago', '10 min', '5 min', 'Now'], label: 'CPU use over the last 15 minutes, now 38 percent' })}</div></div>`)}
<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 14px">
${card(`${small('User')}<div style="font-size: 20px; font-weight: 600; margin-top: 4px">26%</div>`)}
${card(`${small('System')}<div style="font-size: 20px; font-weight: 600; margin-top: 4px">12%</div>`)}
${card(`${small('Cores')}<div style="display: flex; gap: 6px; align-items: center; margin-top: 6px">${coreChip('8 P')}${coreChip('4 E')}</div>`)}
${card(`${small('Top app')}<div style="font-size: 15px; font-weight: 600; margin-top: 6px">Google Chrome · 14.2%</div>`)}
</div>
${card(`<div style="display: flex; flex-direction: column; gap: 8px">
<div style="display: flex; flex-wrap: wrap; align-items: center; gap: 8px"><h2 style="margin: 0; font-size: 13px; font-weight: 600; flex: 1">Apps, busiest first</h2><label for="cpu-find" style="display: flex; align-items: center; gap: 6px; padding: 0 9px; min-height: 28px; border: 1px solid ${t.line}; border-radius: 8px; background: ${t.raised}">${icon('search', 13, t.ink3)}<input id="cpu-find" type="text" placeholder="Name, port or /regex/  ⌘F" style="width: 190px; border: 0; background: transparent; color: ${t.ink}; font: inherit; font-size: 12.5px; outline-offset: 2px"></label>${btn('Columns', { ic: 'columns' })}</div>
<div style="overflow-x: auto"><div style="min-width: 620px; display: flex; flex-direction: column; gap: 2px">
${appRow('Gc', 'Google Chrome', '62 processes', '14.2%', 100, t.cpu, { top: true, open: true, actions: btn('Quit…') })}
${procRow('Tab process', '48211', '4.2%', '1.92 GB')}
${procRow('Google Chrome Helper (GPU)', '48193', '2.8%', '842 MB', pill('GPU'))}
${procRow('Google Chrome', '48190', '1.1%', '412 MB')}
${procRow('Tab process', '48377', '0.3%', '1.31 GB', pill('New'))}
<div style="padding: 2px 0 6px 64px">${btn('Show all 62')}</div>
${appRow('Te', 'Terminal', '14 processes', '9.6%', 68, t.cpu)}
${appRow('Do', 'Docker', '23 processes', '6.1%', 43, t.cpu)}
${appRow('VS', 'VS Code', '31 processes', '4.4%', 31, t.cpu)}
${appRow('OS', 'macOS', '412 processes', '2.4%', 17, t.cpu)}
${appRow('Sa', 'Safari', '9 processes', '1.3%', 9, t.cpu)}
</div></div></div>`)}`);

/* ---------- Inspector ---------- */
const tile = (id, label, v, large = true) => `<div style="display: flex; flex-direction: column; gap: 3px; padding: ${large ? '10px 12px' : '8px 10px'}; background: ${t.raised}; border-radius: 9px; min-width: 0"><span style="display: flex; align-items: center; gap: 5px; font-size: 11.5px; color: ${t.ink2}">${icon(id, 11, t[id])}${label}</span><span style="font-size: ${large ? 18 : 14}px; font-weight: 600">${v}</span></div>`;
const panelHead = (m, title, sub) => `<div style="display: flex; align-items: center; gap: 10px">${mark(m, 36)}<div style="flex: 1; min-width: 0; display: flex; flex-direction: column"><span style="font-size: 15px; font-weight: 600">${title}</span>${small(sub)}</div>${iconBtn('share', 'Copy as text or save as CSV')}${iconBtn('x', 'Close')}</div>`;
const miniProc = (n, pid, cpu, mem, sel = false) => `<div style="display: grid; grid-template-columns: minmax(0, 1fr) 42px 60px; gap: 8px; padding: 6px 8px; border-radius: 7px; font-size: 12.5px; background: ${sel ? t.cpu + '2E' : 'transparent'}"><span style="min-width: 0">${n} ${small(pid)}</span><span style="text-align: right; color: ${t.ink2}">${cpu}</span><span style="text-align: right">${mem}</span></div>`;
const signed = (s) => `<span style="display: inline-flex; gap: 5px; align-items: center">${icon('check', 13, t.good)}${s}</span>`;
const foldRow = (s, n) => `<button type="button" style="display: flex; align-items: center; gap: 7px; padding: 4px 0; border: 0; background: none; color: ${t.ink}; font: inherit; font-size: 13px; font-weight: 500; cursor: pointer; text-align: left">${icon('chevron', 12, t.ink3)}${s} ${pill(n)}</button>`;
const inspector = () => win('cpu', `
<div style="display: flex; flex-wrap: wrap; gap: 14px; align-items: flex-start">
${card(`<div style="display: flex; flex-direction: column; gap: 4px">${section('Apps', small('CPU'))}${[['Gc', 'Google Chrome', '14.2%', 1], ['Te', 'Terminal', '9.6%'], ['Do', 'Docker', '6.1%'], ['VS', 'VS Code', '4.4%'], ['OS', 'macOS', '2.4%'], ['Sa', 'Safari', '1.3%']].map(([m, n, v, s]) => `<div style="display: flex; align-items: center; gap: 8px; padding: 6px; border-radius: 7px; background: ${s ? t.raised : 'transparent'}">${mark(m, 22)}<span style="flex: 1; min-width: 0">${n}</span><span style="font-weight: 600">${v}</span></div>`).join('')}</div>`, 'flex: 1 1 210px')}
${card(`<div style="display: flex; flex-direction: column; gap: 14px">${panelHead('Gc', 'Google Chrome', '62 processes')}
<div style="display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 8px">${tile('memory', 'Memory', '8.60 GB')}${tile('cpu', 'CPU', '14.2%')}</div>
<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(80px, 1fr)); gap: 8px">${tile('battery', 'Power', '4.1 W', false)}${tile('gpu', 'GPU', '3%', false)}${tile('disk', 'Disk', '0 kB/s', false)}${tile('network', 'Network', '1.2 MB/s', false)}</div>
<div style="display: flex; flex-direction: column; gap: 6px">${kv('Version', '156.0.8078.74')}${kv('Bundle ID', 'com.google.Chrome')}${kv('Architecture', 'Universal · arm64 running')}${kv('Signature', signed('Google LLC') + ' ' + small('Team EQHXZ8M8AV'))}${kv('Protection', `<span style="display: inline-flex; gap: 5px; flex-wrap: wrap">${pill('Hardened Runtime')}${pill('Not Sandboxed')}</span>`)}${kv('Started by', 'You, in Finder')}${kv('Location', code('/Applications/Google Chrome.app'))}</div>
${foldRow('Info.plist', '44')}
<div style="display: flex; flex-direction: column; gap: 2px">${section('Processes', btn('Show all 62'))}${miniProc('Tab process', '48211', '4.2%', '1.92 GB')}${miniProc('Chrome Helper (GPU)', '48193', '2.8%', '842 MB', true)}${miniProc('Google Chrome', '48190', '1.1%', '412 MB')}${miniProc('Tab process', '48377', '0.3%', '1.31 GB')}</div>
<div style="display: flex; flex-direction: column; gap: 6px">${section('Ended recently', pill('2'))}<div style="display: flex; justify-content: space-between; gap: 8px; font-size: 12.5px"><span>Tab process ${small('48402')}<br>${small('Ran 1 h 12 min · ended 3 min ago')}</span><span>318 MB</span></div><div style="display: flex; justify-content: space-between; gap: 8px; font-size: 12.5px"><span>Chrome Helper (Plugin) ${small('48395')}<br>${small('Ran 26 min · ended 8 min ago')}</span><span>71 MB</span></div></div>
</div>`, 'flex: 2 1 400px')}
${card(`<div style="display: flex; flex-direction: column; gap: 14px">${panelHead('Gc', 'Chrome Helper (GPU)', 'Google Chrome · PID 48193')}
<div style="display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px">${tile('memory', 'Memory', '842 MB')}${tile('cpu', 'CPU', '2.8%')}${tile('disk', 'Disk', '0 kB/s')}</div>
${section('Identity')}<div style="display: flex; flex-direction: column; gap: 6px">${kv('Process ID', '48193')}${kv('Kind', 'GPU process')}${kv('Parent', link('Google Chrome (48190)'))}${kv('Launched by', link('Google Chrome (48190)'))}${kv('Running for', '6 h 14 min')}${kv('Arguments', code('--type=gpu-process --lang=en-US --gpu-preferences=UAAAAAAAAADgAAAE…'))}</div>
${section('Security')}<div style="display: flex; flex-direction: column; gap: 6px">${kv('Signature', signed('Google LLC'))}${kv('Hardened', 'Yes')}${kv('Sandbox', 'No')}${kv('Entitlements', pill('1'))}</div>
${section('Resource use')}<div style="display: flex; flex-direction: column; gap: 8px">${kv('CPU time', `18 min 2 s<div role="img" aria-label="70 percent user, 30 percent system" style="display: flex; gap: 2px; height: 6px; margin: 5px 0"><div style="flex: 70 1 0; background: ${t.seg[0]}; border-radius: 3px"></div><div style="flex: 30 1 0; background: ${t.seg[1]}; border-radius: 3px"></div></div>${small('User 12 min 37 s · System 5 min 25 s')}`)}${kv('Memory', `842 MB ${small('· 93% of its 904 MB peak')}<div style="margin-top: 5px">${meter(93, t.memory, 6)}</div>`)}${kv('Threads', '31')}${kv('Open files', '142')}</div>
${foldRow('Environment', '31')}
<div style="display: flex; gap: 8px; flex-wrap: wrap">${btn('Quit…')}${btn('Force Quit…')}</div>
</div>`, 'flex: 2 1 400px')}
</div>`);

/* ---------- Projects ---------- */
const projRow = (name, sub, ports, status, kind, mem, action) => `<div style="display: grid; grid-template-columns: 28px minmax(170px, 1.2fr) minmax(110px, 1fr) minmax(150px, 1fr) 70px 74px; align-items: center; gap: 12px; padding: 9px 10px; border-top: 1px solid ${t.line}">${chip('folder', t.projects)}<div style="display: flex; flex-direction: column; min-width: 0"><span style="font-weight: 600">${name}</span>${small(sub)}</div><div style="display: flex; gap: 5px; flex-wrap: wrap">${ports.map(portChip).join('')}</div><div>${pill(status, kind)}</div><span style="text-align: right; font-weight: 600">${mem}</span><span style="display: flex; justify-content: flex-end">${action}</span></div>`;
const row2 = (k, v) => `<div style="display: flex; justify-content: space-between; gap: 16px; font-size: 13px"><span style="color: ${t.ink2}">${k}</span><span style="font-weight: 500; text-align: right">${v}</span></div>`;
const projects = () => win('projects', `
<div style="display: flex; flex-wrap: wrap; align-items: center; gap: 12px; padding: 12px 14px; border-radius: 12px; border: 1px solid ${t.projects}80; background: ${t.projects}1F">${icon('alert', 16, t.projects)}<span style="flex: 1 1 320px"><strong>3 dev servers are running but idle.</strong> Stopping them frees 1.37 GB and ports 3000, 4321, 8000.</span>${btn('Stop All…')}</div>
${card(`<div style="display: flex; flex-direction: column"><div style="padding-bottom: 8px">${section('Projects', small('4 projects · 3.21 GB · 8.1% CPU'))}</div><div style="overflow-x: auto"><div style="min-width: 760px">
${projRow('storefront', 'node · 5 processes', ['5173', '3001'], 'Working', 'good', '1.84 GB', btn('Stop…'))}
${projRow('api-gateway', 'python · 4 processes', ['8000'], 'Up 3 days, barely used', 'warn', '688 MB', btn('Stop…'))}
${projRow('docs-site', 'node · 2 processes', ['4321'], 'Idle 45 min', 'warn', '412 MB', btn('Stop…'))}
${projRow('landing', 'node · 1 process', ['3000'], 'Idle 2 h', 'warn', '270 MB', btn('Stop…'))}
</div></div></div>`)}
<div style="display: flex; justify-content: center; padding: 6px 0">
${card(`<div role="dialog" aria-label="Stop all idle dev servers" style="display: flex; flex-direction: column; gap: 12px">
<h2 style="margin: 0; font-size: 15px; font-weight: 600">Stop 3 idle dev servers?</h2>
<span style="color: ${t.ink2}; line-height: 1.45">api-gateway, docs-site and landing. Each gets a normal stop request first, so it can finish what it is doing.</span>
<div style="display: flex; flex-direction: column; gap: 6px; padding: 10px 12px; border-radius: 9px; background: ${t.raised}">${row2('Processes that close', '7')}${row2('Memory freed', '1.37 GB')}${row2('Ports freed', '3000, 4321, 8000')}${row2('Also connected', 'A browser tab on localhost:3000')}</div>
<div style="display: flex; justify-content: flex-end; gap: 8px">${btn('Keep Running', { kind: 'primary' })}${btn('Stop 3 Servers')}</div>
</div>`, 'width: 100%; max-width: 440px')}
</div>`);

/* ---------- Agents ---------- */
const plan = (pct, c, name, win_, note, red = false) => `<div style="display: flex; align-items: center; gap: 12px; min-width: 0">${ring(pct, c)}<div style="display: flex; flex-direction: column; gap: 1px; min-width: 0"><span style="font-weight: 600">${name}</span>${small(win_)}<span style="display: flex; gap: 4px; align-items: center; font-size: 12px; color: ${red ? t.ink : t.ink3}; font-weight: ${red ? 600 : 400}">${red ? icon('alert', 12, t.strain) : ''}${note}</span></div></div>`;
const childRow = (n, kind, mem) => `<div style="display: flex; justify-content: space-between; gap: 12px; padding: 5px 0; font-size: 12.5px"><span><span style="font-weight: 500">${n}</span> ${small(kind)}</span><span style="color: ${t.ink2}">${mem}</span></div>`;
const session = (m, title, st, kind, sub, mem, kids, n, button) => card(`<div style="display: flex; flex-direction: column; gap: 10px">
<div style="display: flex; flex-wrap: wrap; align-items: center; gap: 10px">${mark(m, 32)}<div style="flex: 1 1 280px; display: flex; flex-direction: column; gap: 2px; min-width: 0"><span style="display: flex; flex-wrap: wrap; align-items: center; gap: 8px"><span style="font-weight: 600">${title}</span>${pill(st, kind)}</span>${small(sub)}</div><span style="font-size: 14px; font-weight: 600">${mem}</span>${button}</div>
${kids.length ? `<div style="display: flex; flex-direction: column; padding: 6px 0 0 12px; margin-left: 15px; border-left: 2px solid ${t.agents}">${small('Started · ' + n)}${kids.join('')}</div>` : small('Nothing started')}</div>`);
const agents = () => win('agents', `
${card(`<div style="display: flex; flex-direction: column; gap: 14px">${section('Plans', small('Checked 2 min ago'))}<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 16px">
${plan(26, t.seg[1], 'Claude', 'This week', 'On pace, 10% to spare')}${plan(11, t.seg[2], 'Codex', '5-hour window', 'Resets in 3 h 12 min')}${plan(72, t.strain, 'GitHub Copilot', 'Premium requests', 'Runs out in 4 days at this rate', true)}${plan(18, t.seg[0], 'Cursor', 'This billing cycle', 'Resets in 12 days')}
</div></div>`)}
${section('Sessions', small('3 sessions · 2.02 GB · 6.4% CPU'))}
${session('CC', 'Checkout flow tests', 'Working', 'good', 'Claude Code · storefront · running 48 min', '1.56 GB', [childRow('vitest', 'tests · 5.1% CPU', '524 MB'), childRow('node', 'dev server · :5173', '243 MB'), childRow('Chrome for Testing', 'browser', '309 MB'), childRow('mcp-server-fetch', 'tool server', '64 MB')], 4, btn('Quit Session…'))}
${session('Cx', 'Rewrite install guide', 'Waiting for you · 6 min', 'wait', 'Codex · docs-site · running 22 min', '290 MB', [], 0, btn('Quit Session…'))}
${session('Ai', 'aider', 'Idle 1 h 10 min', 'plain', 'Aider · api-gateway · running 3 h 4 min', '168 MB', [], 0, btn('Quit Session…'))}
${card(`<div style="display: flex; align-items: center; gap: 10px">${icon('lock', 15, t.ink3)}<span style="color: ${t.ink2}">Details reads each session's process, folder and children. It does not read code or conversations.</span></div>`)}`);

/* ---------- Over time ---------- */
const seg = (items, on) => `<div style="display: inline-flex; flex-wrap: wrap; padding: 2px; border-radius: 9px; background: ${t.raised}">${items.map((s) => `<button type="button" style="padding: 4px 10px; border: 0; border-radius: 7px; background: ${s === on ? t.card : 'transparent'}; color: ${s === on ? t.ink : t.ink2}; font: inherit; font-size: 12px; font-weight: ${s === on ? 600 : 400}; cursor: pointer">${s}</button>`).join('')}</div>`;
const rank = (n, v, pct, c) => `<div style="display: grid; grid-template-columns: 130px minmax(60px, 1fr) 56px; gap: 10px; align-items: center; font-size: 12.5px"><span>${n}</span>${meter(pct, c, 6)}<span style="text-align: right; font-weight: 500">${v}</span></div>`;
const peak = (id, l, v, when, app) => card(`<div style="display: flex; flex-direction: column; gap: 5px"><span style="display: flex; align-items: center; gap: 6px; font-size: 12px; color: ${t.ink2}">${icon(id, 12, t[id] ?? t.cpu)}${l}</span><span style="font-size: 22px; font-weight: 600">${v}</span>${small(when)}${small(app)}</div>`);
const days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const overTime = () => win('overview', `
<div style="display: flex; flex-wrap: wrap; align-items: center; gap: 10px"><h1 style="margin: 0; font-size: 17px; font-weight: 600; flex: 1 1 150px">Over Time</h1>${seg(['CPU', 'Memory', 'Energy', 'Disk', 'Network'], 'CPU')}${seg(['1 h', '12 h', '24 h', '7 d', '30 d', '90 d', '6 mo', '1 y'], '7 d')}${btn('Export CSV')}</div>
${card(`<div style="display: flex; flex-direction: column; gap: 10px">${section('CPU, average of each 6 hours', small('Recording since 12 October'))}${columns(series(28, 1.2, 36, 18).map((v, i) => (i % 4 === 0 ? v * 0.35 : v)), t.cpu, { xs: days.map((d, i) => [i * 4 + 1.5, d]), label: 'Average CPU per 6 hours for the last 7 days, highest on Tuesday' })}</div>`)}
<div style="display: flex; flex-wrap: wrap; gap: 14px; align-items: flex-start">
${card(`<div style="display: flex; flex-direction: column; gap: 9px">${section('Used the most, Tuesday 14:00 to 15:00')}${rank('Docker', '31.2%', 100, t.cpu)}${rank('Terminal', '22.7%', 73, t.cpu)}${rank('Google Chrome', '18.4%', 59, t.cpu)}${rank('VS Code', '7.9%', 25, t.cpu)}${rank('Xcode', '5.1%', 16, t.cpu)}</div>`, 'flex: 1 1 380px')}
${card(`<div style="display: flex; flex-direction: column; gap: 10px">${section('At a limit', small('3 stretches this week'))}${[['CPU at 90% or more', 'Tue 14:02 to 14:21', '19 min'], ['Memory under pressure', 'Thu 10:40 to 10:52', '12 min'], ['Thermal slowdown', 'Tue 14:09 to 14:14', '5 min']].map(([a, b, c]) => `<div style="display: flex; justify-content: space-between; gap: 10px; font-size: 12.5px"><span><span style="font-weight: 500">${a}</span><br>${small(b)}</span><span style="font-weight: 600">${c}</span></div>`).join('')}</div>`, 'flex: 1 1 320px')}
</div>
${section('Peaks this week')}
<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 14px">${peak('cpu', 'Highest CPU', '94%', 'Tue 14:05', 'Docker, Terminal')}${peak('memory', 'Highest memory pressure', 'Critical', 'Thu 10:44', 'Google Chrome, Docker')}${peak('therm', 'Hottest', '96°C', 'Tue 14:11', 'Docker')}${peak('battery', 'Most power', '41.3 W', 'Tue 14:06', 'Docker, VS Code')}</div>`);

/* ---------- Sound ---------- */
const slider = (id, label, v, c) => `<input id="${id}" aria-label="${label}" type="range" min="0" max="100" value="${v}" style="width: 100%; accent-color: ${c}; margin: 0">`;
const soundRow = (id, m, name, status, kind, v, level, muted) => `<div style="display: grid; grid-template-columns: 28px minmax(130px, 1fr) 130px minmax(160px, 1.4fr) 44px 110px; gap: 12px; align-items: center; padding: 9px 4px; border-top: 1px solid ${t.line}">${mark(m, 26)}<span style="font-weight: 500">${name}</span><span>${pill(status, kind)}</span>${slider(id, 'Volume for ' + name, v, t.sound)}<span style="text-align: right; font-weight: 500">${muted ? 'Muted' : v + '%'}</span><div>${meter(level, t.sound, 6)}</div></div>`;
const sound = () => win('sound', `
<div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: 14px">
${card(`<div style="display: flex; flex-direction: column; gap: 10px">${section('Output', btn('MacBook Pro Speakers', { ic: 'down' }))}<div style="display: grid; grid-template-columns: minmax(0, 1fr) 40px; gap: 10px; align-items: center">${slider('out', 'Output volume', 62, t.sound)}<span style="text-align: right; font-weight: 600">62%</span></div></div>`)}
${card(`<div style="display: flex; flex-direction: column; gap: 10px">${section('Microphone', btn('MacBook Pro Microphone', { ic: 'down' }))}<div style="display: flex; align-items: center; gap: 8px">${icon('mic', 14, t.sound)}<span style="flex: 1">Listening right now: <strong>Zoom</strong></span>${pill('In use', 'wait')}</div><div>${meter(38, t.sound, 6)}</div></div>`)}
</div>
${card(`<div style="display: flex; flex-direction: column"><div style="padding-bottom: 8px">${section('Apps with sound open', small('Volume is remembered per app. Nothing is recorded.'))}</div><div style="overflow-x: auto"><div style="min-width: 740px">
${soundRow('v-music', 'Mu', 'Music', 'Playing', 'good', 70, 64)}
${soundRow('v-zoom', 'Zo', 'Zoom', 'Playing', 'good', 100, 41)}
${soundRow('v-chrome', 'Gc', 'Google Chrome', 'Playing', 'good', 45, 22)}
${soundRow('v-slack', 'Sl', 'Slack', 'Silent', 'plain', 30, 0)}
${soundRow('v-safari', 'Sa', 'Safari', 'Silent', 'plain', 0, 0, true)}
</div></div></div>`)}`);

/* ---------- Menu bar and dropdown ---------- */
const barItem = (ic, text, { red = false, c, sp } = {}) => `<span style="display: inline-flex; align-items: center; gap: 4px; font-size: 12.5px; font-weight: 500; color: ${red ? t.strain : t.ink}; white-space: nowrap">${ic ? icon(ic, 13, red ? t.strain : t.ink) : ''}${text}${sp ? spark(sp, red ? t.strain : c ?? t.ink, 44, 16) : ''}</span>`;
const menuStrip = (items, extra = '') => `<div style="display: flex; flex-wrap: wrap; align-items: center; justify-content: flex-end; gap: 13px; padding: 5px 12px; min-height: 26px; box-sizing: border-box; background: #20242C; border-bottom: 1px solid ${t.line}; ${extra}">${items.join('')}</div>`;
const ddTabs = (on) => `<div style="display: flex; justify-content: space-between; gap: 1px; padding: 3px; background: ${t.raised}; border-radius: 10px">${['overview', 'cpu', 'memory', 'disk', 'network', 'gpu', 'battery', 'sound', 'bluetooth', 'projects', 'agents'].map((id) => `<button type="button" aria-label="${id}" style="display: inline-flex; width: 28px; height: 26px; align-items: center; justify-content: center; padding: 0; border: 0; border-radius: 7px; cursor: pointer; background: ${id === on ? (t[id] ?? t.cpu) + '40' : 'transparent'}">${icon(id, 13, id === on ? t[id] ?? t.cpu : t.ink3)}</button>`).join('')}</div>`;
const ddTile = (id, l, v, u, data) => `<div style="display: flex; flex-direction: column; gap: 5px; padding: 9px 10px; background: ${t.raised}; border-radius: 9px; min-width: 0"><span style="display: flex; align-items: center; gap: 5px; font-size: 11.5px; color: ${t.ink2}">${icon(id, 11, t[id])}${l}</span><div style="display: flex; align-items: flex-end; justify-content: space-between; gap: 6px"><span style="font-size: 19px; font-weight: 600; white-space: nowrap">${v}<span style="font-size: 11px; color: ${t.ink2}; font-weight: 400"> ${u}</span></span>${spark(data, t[id], 58, 20)}</div></div>`;
const ddApp = (m, n, v, pct, c) => `<div style="display: grid; grid-template-columns: 20px minmax(0, 1fr) 54px 46px; gap: 8px; align-items: center; font-size: 12.5px">${mark(m, 20)}<span>${n}</span>${meter(pct, c)}<span style="text-align: right; font-weight: 500">${v}</span></div>`;
const ddFoot = () => `<div style="display: flex; gap: 6px">${btn('Open Details', { extra: 'flex: 1' })}${iconBtn('gear', 'Settings')}${iconBtn('power', 'Quit Details')}</div>`;
const dd = (body) => `<div style="width: 370px; max-width: 100%; box-sizing: border-box; display: flex; flex-direction: column; gap: 10px; padding: 10px; background: ${t.card}; border: 1px solid ${t.line}; border-radius: 14px">${body}</div>`;
const ddOverview = () => dd(`${ddTabs('overview')}<div style="display: flex; justify-content: space-between; align-items: baseline; padding: 0 2px"><span style="font-weight: 600">Overview</span>${small('Up 6 days 4 hours')}</div>
<div style="display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px">${ddTile('cpu', 'CPU', '38', '%', series(20, 1, 40, 12))}${ddTile('memory', 'Memory', '35.7', 'GB', series(20, 2, 74, 2))}${ddTile('gpu', 'GPU', '46', '%', series(20, 3, 42, 13))}${ddTile('network', 'Down', '5.3', 'MB/s', series(20, 5, 28, 22))}${ddTile('disk', 'Disk free', '412', 'GB', series(20, 4, 24, 20))}${ddTile('battery', 'Battery', '63', '%', series(20, 6, 68, 3))}</div>
<div style="display: flex; flex-direction: column; gap: 7px; padding: 2px">${small('Busiest right now')}${ddApp('Gc', 'Google Chrome', '14.2%', 100, t.cpu)}${ddApp('Te', 'Terminal', '9.6%', 68, t.cpu)}${ddApp('Do', 'Docker', '6.1%', 43, t.cpu)}</div>${ddFoot()}`);
const ddStrain = () => dd(`<button type="button" style="display: flex; align-items: center; gap: 7px; padding: 7px 10px; border-radius: 9px; border: 0; background: ${t.strain}2E; color: ${t.ink}; font: inherit; font-size: 12.5px; font-weight: 600; cursor: pointer; text-align: left">${icon('alert', 13, t.strain)}CPU under strain · 94% for 6 min</button>${ddTabs('cpu')}
<div style="display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 8px 10px; background: ${t.raised}; border-radius: 10px"><div>${big('94', '%', 30)}${small('Apple M4 Pro')}</div>${spark(series(20, 2, 60, 8).concat([70, 82, 90, 94, 93, 94]), t.strain, 130, 40)}</div>
<div style="display: flex; flex-direction: column; gap: 5px; padding: 0 2px">${row2('User', '71%')}${row2('System', '23%')}${row2('Load', '13.42')}</div>
<div style="display: flex; flex-direction: column; gap: 7px; padding: 2px">${small('Top apps')}${ddApp('Do', 'Docker', '48.7%', 100, t.cpu)}${ddApp('Te', 'Terminal', '27.3%', 56, t.cpu)}${ddApp('Gc', 'Google Chrome', '11.2%', 23, t.cpu)}</div>${ddFoot()}`);
const styleRow = (l, content) => `<div style="display: flex; align-items: center; gap: 14px; padding: 9px 0; border-top: 1px solid ${t.line}; flex-wrap: wrap"><span style="width: 72px; color: ${t.ink2}; font-size: 12.5px">${l}</span>${content}</div>`;
const frame = (x) => `<span style="display: inline-flex; padding: 1px 3px; border: 1px solid ${t.ink3}; border-radius: 4px">${x}</span>`;
const menuBoard = () => { const S = series(14, 1, 40, 18);
  return `<div style="background: ${t.bg}; color: ${t.ink}; font-family: ${FONT}; font-variant-numeric: tabular-nums; font-size: 13px; padding: 24px; box-sizing: border-box; display: flex; flex-wrap: wrap; gap: 24px; align-items: flex-start; min-height: 100%">
<div style="flex: 1 1 380px; display: flex; flex-direction: column; gap: 18px; min-width: 0">
<div style="display: flex; flex-direction: column; gap: 8px"><h2 style="margin: 0; font-size: 14px; font-weight: 600">Normal</h2><div style="border: 1px solid ${t.line}; border-radius: 9px; overflow: hidden">${menuStrip([barItem('cpu', '38%'), barItem('memory', '35.7 GB'), barItem('network', '↓5.3 MB/s ↑240 kB/s'), barItem('therm', '53°')])}</div></div>
<div style="display: flex; flex-direction: column; gap: 8px"><h2 style="margin: 0; font-size: 14px; font-weight: 600">Under strain, red until it passes</h2><div style="border: 1px solid ${t.line}; border-radius: 9px; overflow: hidden">${menuStrip([barItem('cpu', '94%', { red: true }), barItem('memory', '35.7 GB'), barItem('network', '↓5.3 MB/s ↑240 kB/s'), barItem('therm', '91°', { red: true })])}</div></div>
${card(`<h2 style="margin: 0 0 8px; font-size: 14px; font-weight: 600">One reading, four styles</h2>
${styleRow('Value', barItem('cpu', '38%'))}${styleRow('Graph', barItem('', '', { sp: S, c: t.cpu }))}${styleRow('Both', barItem('cpu', '38%', { sp: S, c: t.cpu }))}${styleRow('Framed', frame(barItem('', '', { sp: S, c: t.cpu })))}
${styleRow('Compact', `<span style="font-size: 10.5px; font-weight: 500; line-height: 1.05; text-align: center">38%<br>CPU</span>`)}${styleRow('No icon', barItem('', '38%'))}`)}
${card(`<div style="display: flex; flex-direction: column; gap: 6px"><span style="font-weight: 600">Plans in the menu bar</span><div>${barItem('', 'Cl 26 · Cx 11 · Co 72')}</div>${small('Off until turned on. Provider marks are still to design.')}</div>`)}
</div>
<div style="flex: 0 1 370px; min-width: 0; display: flex; flex-direction: column; gap: 8px"><h2 style="margin: 0; font-size: 14px; font-weight: 600">Dropdown, Overview tiles</h2>${ddOverview()}</div>
<div style="flex: 0 1 370px; min-width: 0; display: flex; flex-direction: column; gap: 8px"><h2 style="margin: 0; font-size: 14px; font-weight: 600">Dropdown, CPU page under strain</h2>${ddStrain()}</div>
</div>`; };

/* ---------- Approval window ---------- */
const approval = () => `<div style="background: ${t.bg}; color: ${t.ink}; font-family: ${FONT}; font-variant-numeric: tabular-nums; font-size: 13px; padding: 24px; box-sizing: border-box; min-height: 100%; display: flex; justify-content: center; align-items: center">
<div role="dialog" aria-label="Assistant request to quit" style="width: 460px; max-width: 100%; background: ${t.card}; border: 1px solid ${t.line}; border-radius: 12px; overflow: hidden">
<div style="display: flex; align-items: center; gap: 12px; padding: 10px 14px; border-bottom: 1px solid ${t.line}">${lights}<span style="font-size: 12.5px; font-weight: 500; color: ${t.ink2}">Details</span></div>
<div style="display: flex; flex-direction: column; gap: 14px; padding: 18px">
<div style="display: flex; gap: 12px; align-items: center">${mark('CC', 38)}<div style="display: flex; flex-direction: column; gap: 2px"><span style="font-size: 15px; font-weight: 600">Claude Code wants to quit docs-site</span>${small('Asked just now, from a session in storefront')}</div></div>
<div style="display: flex; flex-direction: column; gap: 7px; padding: 10px 12px; border-radius: 9px; background: ${t.raised}">${row2('Target', code('astro dev') + ' · PID 51204')}${row2('Processes that close', '2')}${row2('Memory freed', '412 MB')}${row2('Port freed', '4321')}${row2('Connected to it', 'Nothing')}</div>
<div style="display: flex; gap: 10px; align-items: center">${pill('Safe', 'good')}<span style="color: ${t.ink2}; font-size: 12.5px">Idle for 45 minutes, with no unsaved work found.</span></div>
<div style="display: flex; flex-direction: column; gap: 5px">${meter(70, t.ink3)}${small('No answer in 42 seconds counts as Decline.')}</div>
<div style="display: flex; gap: 8px; justify-content: flex-end; flex-wrap: wrap">${btn('Decline', { kind: 'primary' })}${btn('Quit docs-site')}</div>
<span style="font-size: 12px; color: ${t.ink3}">Force Quit always needs you. This request is saved in the request log.</span>
</div></div></div>`;

/* ---------- Settings ---------- */
const field = (id, label, v, unit) => `<label for="${id}" style="display: inline-flex; align-items: center; gap: 6px; font-size: 12.5px; color: ${t.ink2}">${label}<input id="${id}" type="text" value="${v}" style="width: 44px; min-height: 26px; padding: 2px 6px; box-sizing: border-box; border: 1px solid ${t.line}; border-radius: 6px; background: ${t.raised}; color: ${t.ink}; font: inherit; text-align: right">${unit}</label>`;
const toggle = (on) => `<span aria-hidden="true" style="display: inline-flex; width: 32px; height: 18px; padding: 2px; box-sizing: border-box; border-radius: 9px; background: ${on ? t.good : t.line}; justify-content: ${on ? 'flex-end' : 'flex-start'}; flex: none"><span style="width: 14px; height: 14px; border-radius: 7px; background: #FFFFFF"></span></span>`;
const ruleRow = (title, fields, on = true) => `<div style="display: flex; flex-wrap: wrap; align-items: center; gap: 10px 14px; padding: 10px 0; border-top: 1px solid ${t.line}"><span style="flex: 1 1 200px; font-weight: 500">${title}</span><span style="display: flex; flex-wrap: wrap; gap: 10px">${fields}</span>${toggle(on)}</div>`;
const settings = () => `<div style="background: ${t.bg}; color: ${t.ink}; font-family: ${FONT}; font-variant-numeric: tabular-nums; font-size: 13px; box-sizing: border-box; min-height: 100%; display: flex; flex-wrap: wrap">
<nav aria-label="Settings pages" style="flex: 1 1 180px; display: flex; flex-direction: column; gap: 2px; padding: 12px; border-right: 1px solid ${t.line}; box-sizing: border-box"><div style="padding: 2px 4px 12px">${lights}</div>${['General', 'Layout', 'Customize', 'Insights', 'Integrations', 'About'].map((p) => `<button type="button" style="text-align: left; padding: 6px 10px; border: 0; border-radius: 7px; background: ${p === 'Insights' ? t.cpu + '33' : 'transparent'}; color: ${t.ink}; font: inherit; font-weight: ${p === 'Insights' ? 600 : 400}; cursor: pointer">${p}</button>`).join('')}</nav>
<div style="flex: 999 1 560px; min-width: 0; display: flex; flex-direction: column; gap: 14px; padding: 18px">
<h1 style="margin: 0; font-size: 18px; font-weight: 600">Insights</h1>
${card(`<div style="display: flex; align-items: center; gap: 12px"><div style="flex: 1; display: flex; flex-direction: column; gap: 2px"><span style="font-weight: 600">Notifications</span>${small('Details tells you once, then leaves that app alone for an hour. It never quits anything because of an alert.')}</div>${toggle(true)}</div>`)}
${card(`<div style="display: flex; flex-direction: column"><div style="padding-bottom: 8px">${section('Tell me about patterns')}</div>
${ruleRow('An app stays busy', field('r1', 'CPU over', '70', '%') + field('r2', 'for', '10', 'min'))}
${ruleRow('An app keeps growing', field('r3', 'Memory up', '2', 'GB') + field('r4', 'in', '60', 'min'))}
${ruleRow('Disk or network burst', field('r5', 'Above usual by', '10', 'times'))}
${ruleRow('The Mac stays hot', field('r6', 'Over', '90', '°C') + field('r7', 'for', '5', 'min'))}</div>`)}
<div style="display: flex; flex-wrap: wrap; gap: 14px">
${card(`<div style="display: flex; flex-direction: column; gap: 8px">${section('Tell me when', btn('Add…'))}<div style="display: flex; justify-content: space-between; gap: 10px; font-size: 12.5px"><span>${'Docker'} ${small('Memory over 6 GB for 5 min')}</span>${iconBtn('x', 'Remove rule for Docker')}</div><div style="display: flex; justify-content: space-between; gap: 10px; font-size: 12.5px"><span>${'Xcode'} ${small('CPU over 50% for 15 min')}</span>${iconBtn('x', 'Remove rule for Xcode')}</div></div>`, 'flex: 1 1 300px')}
${card(`<div style="display: flex; flex-direction: column; gap: 8px">${section("Don't tell me about")}<div style="display: flex; justify-content: space-between; align-items: center; gap: 10px; font-size: 12.5px"><span>Time Machine ${small('Always')}</span>${btn('Tell Me Again')}</div><div style="display: flex; justify-content: space-between; align-items: center; gap: 10px; font-size: 12.5px"><span>Spotlight ${small('Until tomorrow')}</span>${btn('Tell Me Again')}</div></div>`, 'flex: 1 1 300px')}
</div></div></div>`;

/* ---------- First run and gaps (designed here, not on the reference site) ---------- */
const stateCard = (title, body) => `<div style="flex: 1 1 340px; min-width: 0; display: flex; flex-direction: column; gap: 8px"><h2 style="margin: 0; font-size: 14px; font-weight: 600">${title}</h2>${body}</div>`;
const states = () => `<div style="background: ${t.bg}; color: ${t.ink}; font-family: ${FONT}; font-variant-numeric: tabular-nums; font-size: 13px; padding: 24px; box-sizing: border-box; min-height: 100%; display: flex; flex-wrap: wrap; gap: 20px; align-items: flex-start">
${stateCard('First run', card(`<div style="display: flex; flex-direction: column; gap: 12px"><span style="font-size: 18px; font-weight: 600">See which app is doing it</span><span style="color: ${t.ink2}; line-height: 1.45">Details works now, with your own apps. Everything stays on this Mac: no account and no analytics.</span>
<div style="display: flex; gap: 10px; align-items: flex-start">${icon('check', 15, t.good)}<span><strong>Apps, memory, disk and network</strong><br>${small('Ready')}</span></div>
<div style="display: flex; gap: 10px; align-items: flex-start">${icon('lock', 15, t.ink3)}<span style="flex: 1"><strong>System processes, temperatures and fans</strong><br>${small('Need a small helper, which needs your password. You can add it later.')}</span>${btn('Add…')}</div>
<div style="display: flex; gap: 10px; align-items: flex-start">${icon('lock', 15, t.ink3)}<span style="flex: 1"><strong>Plan limits and assistants</strong><br>${small('Off until you turn them on in Integrations.')}</span></div>
${btn('Start Using Details', { kind: 'primary' })}</div>`))}
${stateCard('A reading the Mac will not give', card(`<div style="display: flex; flex-direction: column; gap: 10px"><div style="display: flex; align-items: center; gap: 8px">${chip('gpu', t.gpu)}<span style="font-weight: 600; flex: 1">GPU</span>${pill('Unavailable')}</div><div style="font-size: 28px; font-weight: 600; color: ${t.ink3}">—</div><span style="color: ${t.ink2}; line-height: 1.45">This chip has not reported GPU use since macOS 27.2. Details shows a dash here, because 0% would mean the GPU is idle.</span>${btn('Check Again')}</div>`))}
${stateCard('No history yet', card(`<div style="display: flex; flex-direction: column; gap: 10px">${section('Over Time', small('Recording for 12 min'))}<div style="height: 120px; border: 1px dashed ${t.line}; border-radius: 9px; display: flex; align-items: center; justify-content: center; text-align: center; padding: 12px; box-sizing: border-box; color: ${t.ink2}">The 7-day view fills in as Details records.<br>Try 1 h for what it has so far.</div>${btn('Show 1 Hour')}</div>`))}
${stateCard('Without the helper', card(`<div style="display: flex; flex-direction: column; gap: 8px">${kv('Process ID', '402')}${kv('User', 'root')}${kv('Arguments', `<span style="color: ${t.ink2}">Unavailable: this process belongs to another user.</span>`)}${kv('Environment', `<span style="color: ${t.ink2}">Unavailable: macOS does not share it.</span>`)}<div style="display: flex; justify-content: flex-end">${btn('Add Helper…')}</div></div>`))}
</div>`;

const BOARDS = [
  { h: 800, file: 'Main.dc.html', title: 'Overview', page: overview, fill: true },
  { h: 800, file: 'OverviewLight.dc.html', title: 'Overview, light', page: overview, theme: LIGHT, fill: true },
  { h: 990, file: 'CPU.dc.html', title: 'CPU tab, app unfolded', page: cpuTab, fill: true },
  { h: 860, file: 'Inspector.dc.html', title: 'App card and process inspector', page: inspector, fill: true },
  { h: 650, file: 'MenuBar.dc.html', title: 'Menu bar and dropdown', page: menuBoard, fill: true },
  { h: 700, file: 'Projects.dc.html', title: 'Projects, with Stop All confirmation', page: projects, fill: true },
  { h: 740, file: 'Agents.dc.html', title: 'Agents and plans', page: agents, fill: true },
  { h: 760, file: 'OverTime.dc.html', title: 'Over Time and Peaks', page: overTime, fill: true },
  { h: 510, file: 'Sound.dc.html', title: 'Sound', page: sound, fill: true },
  { file: 'Approval.dc.html', title: 'Assistant approval window', page: approval, w: 620, h: 560, fixed: true },
  { h: 560, file: 'Settings.dc.html', title: 'Settings, Insights page', page: settings, fill: true },
  { h: 620, file: 'States.dc.html', title: 'First run and honest gaps', page: states, fill: true },
];
const render = (b, rootHeight = '') => { t = b.theme ?? DARK; return b.page().replaceAll('min-height: 100%', rootHeight); };

const here = dirname(fileURLToPath(import.meta.url));
const local = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Details mockups</title></head>
<body style="margin: 0; background: #0B0D10; color: #F0F2F5; font-family: ${FONT}">
<main style="max-width: 1280px; margin: 0 auto; padding: 24px 16px; display: flex; flex-direction: column; gap: 32px">
<header><h1 style="margin: 0 0 6px; font-size: 24px">Details mockups</h1><p style="margin: 0; color: #AEB5BF">All numbers and app rows are sample data. App marks are placeholders. See ../ui-ux.md.</p></header>
${BOARDS.map((b, i) => `<section id="${b.file.split('.')[0]}" data-board="${b.file}"><h2 style="margin: 0 0 10px; font-size: 14px; color: #AEB5BF">${i + 1}. ${b.title}</h2><div data-frame style="border-radius: 12px; overflow: hidden; border: 1px solid #2E333B${b.fixed ? `; max-width: ${b.w}px` : ''}">${render(b)}</div></section>`).join('\n')}
</main></body></html>`;
writeFileSync(join(here, 'index.html'), local);

const canvasDir = process.argv[2];
if (canvasDir) {
  mkdirSync(join(canvasDir, 'project'), { recursive: true });
  for (const b of BOARDS) {
    const w = 1280, h = b.h ?? 900;
    writeFileSync(join(canvasDir, 'project', b.file), `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>${b.title}</title>
<script src="./support.js"></script>
</head>
<body>
<x-dc>
<helmet>
<style>
body{margin:0}
a{color:#6DAEFF}a:hover{color:#9CC8FF}
</style>
</helmet>
${render(b, 'min-height: 100vh')}
</x-dc>
<script type="text/x-dc" data-dc-script data-props='{"$preview":{"width":${b.w ?? w},"height":${h}}}'>
class Component extends DCLogic {
renderVals() {
return {};
}
}
</script>
</body>
</html>
`);
  }
}
if (canvasDir) {
  const boards = {}; let y = 0;
  for (let i = 0; i < BOARDS.length; i += 3) {
    const row = BOARDS.slice(i, i + 3);
    row.forEach((b, j) => { boards[b.file] = { x: j * 1360, y, w: b.w ?? 1280, h: b.h, title: b.title, ...(b.fill ? { expand: 'fill' } : {}) }; });
    y += Math.max(...row.map((b) => b.h)) + 120;
  }
  writeFileSync(join(canvasDir, 'project', 'canvas.json'), JSON.stringify({
    v: 3, createdOnFiles: { v: 1, at: new Date().toISOString() }, title: 'Details mockups', launch: { view: 'canvas' }, pages: [], boards,
    order: BOARDS.map((b) => b.file),
    notes: { sample: { x: 0, y: -210, w: 520, size: 'm', fill: 'gray', text: 'Every number and app row is sample data. App marks are placeholders, not real logos. Tabs with no board here (Memory, Disk, Network, GPU, Neural, Battery, Bluetooth, Containers) reuse the CPU layout.' } },
    designSystems: [],
  }, null, 1));
}
