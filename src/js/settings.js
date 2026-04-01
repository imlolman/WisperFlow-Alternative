const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let capturing = null;
let platform = 'macos';

const CODE_MAP = {
  AltLeft:'key:Alt_L', AltRight:'key:Alt_R',
  ControlLeft:'key:Control_L', ControlRight:'key:Control_R',
  ShiftLeft:'key:Shift_L', ShiftRight:'key:Shift_R',
  MetaLeft:'key:Super_L', MetaRight:'key:Super_R',
  Space:'key:space', Enter:'key:Return', Escape:'key:Escape',
  Backspace:'key:BackSpace', Tab:'key:Tab', CapsLock:'key:Caps_Lock',
  Delete:'key:Delete',
  PageDown:'key:Page_Down', PageUp:'key:Page_Up',
  Home:'key:Home', End:'key:End',
  ArrowUp:'key:Up', ArrowDown:'key:Down',
  ArrowLeft:'key:Left', ArrowRight:'key:Right',
};
for (let i = 1; i <= 12; i++) CODE_MAP['F' + i] = 'key:F' + i;
for (let i = 0; i <= 9; i++) CODE_MAP['Digit' + i] = 'key:' + i;
'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').forEach(c => {
  CODE_MAP['Key' + c] = 'key:' + c.toLowerCase();
});

function keyDisp(code) {
  if (platform === 'macos') {
    const m = {
      ControlLeft: '\u2303', ControlRight: '\u2303',
      ShiftLeft: '\u21e7', ShiftRight: '\u21e7',
      AltLeft: '\u2325', AltRight: '\u2325',
      MetaLeft: '\u2318', MetaRight: '\u2318',
      Escape: 'Esc', Enter: '\u21a9', Backspace: '\u232b',
      Delete: '\u2326', Tab: '\u21e5', Space: '\u2423',
    };
    if (m[code]) return m[code];
  } else {
    const m = {
      ControlLeft: 'Ctrl', ControlRight: 'Ctrl',
      ShiftLeft: 'Shift', ShiftRight: 'Shift',
      AltLeft: 'Alt', AltRight: 'Alt',
      MetaLeft: platform === 'windows' ? 'Win' : 'Super',
      MetaRight: platform === 'windows' ? 'Win' : 'Super',
      Escape: 'Esc', Enter: 'Enter', Backspace: 'Backspace',
      Delete: 'Del', Tab: 'Tab', Space: 'Space',
    };
    if (m[code]) return m[code];
  }
  if (code.startsWith('Key')) return code.slice(3);
  if (code.startsWith('Digit')) return code.slice(5);
  if (code.startsWith('F') && code.length <= 3) return code;
  return code;
}

async function init() {
  platform = await invoke('get_platform');
  
  const cfg = await invoke('get_config');
  document.getElementById('holdDisp').textContent = cfg._hold_display || 'Not set';
  document.getElementById('toggleDisp').textContent = cfg._toggle_display || 'Not set';
  document.getElementById('pasteDisp').textContent = cfg._paste_display || 'Not set';
  document.getElementById('chkDock').checked = cfg.hide_dock_icon;
  document.getElementById('chkMenu').checked = cfg.hide_menu_icon;
  document.getElementById('chkLogin').checked = cfg.start_on_login;
  
  if (platform !== 'macos') {
    document.getElementById('dockRow').style.display = 'none';
    document.getElementById('trayLabel').textContent = 'Auto-hide system tray icon';
  }
  
  loadMics(cfg.mic_device);
  loadHistory();
}

async function loadMics(selected) {
  const mics = await invoke('get_microphones');
  const sel = document.getElementById('micSel');
  sel.innerHTML = '<option value="">Default</option>';
  mics.forEach(m => {
    const opt = document.createElement('option');
    opt.value = m.name;
    opt.textContent = m.name + (m.is_default ? ' (Default)' : '');
    if (m.name === selected) opt.selected = true;
    sel.appendChild(opt);
  });
  sel.onchange = () => {
    invoke('save_mic', { device: sel.value || null });
  };
}

async function loadHistory() {
  const hist = await invoke('get_history');
  const el = document.getElementById('hlist');
  if (!hist.length) {
    el.innerHTML = '<div style="color:var(--text2);font-size:12px;padding:6px 0">No history yet</div>';
    return;
  }
  el.innerHTML = '';
  [...hist].reverse().forEach(h => {
    const row = document.createElement('div');
    row.className = 'hrow';
    const time = h.timestamp.split(' ')[1] || '';
    const hm = time.split(':').slice(0, 2).join(':');
    row.innerHTML = `<span class="htime">${hm}</span><span class="htxt">${esc(h.text)}</span><button class="hcopy" onclick="cp(this,'${esc2(h.text)}')">Copy</button>`;
    el.appendChild(row);
  });
}

function esc(s) { return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;'); }
function esc2(s) { return s.replace(/\\/g,'\\\\').replace(/'/g,"\\'").replace(/\n/g,'\\n'); }

async function cp(btn, text) {
  await invoke('copy_text', { text });
  btn.textContent = '\u2713';
  setTimeout(() => btn.textContent = 'Copy', 1000);
}

function tog(field) {
  const map = { hide_dock_icon: 'chkDock', hide_menu_icon: 'chkMenu', start_on_login: 'chkLogin' };
  const chk = document.getElementById(map[field]);
  chk.checked = !chk.checked;
  invoke('save_field', { field, value: chk.checked });
}

function capture(which) {
  if (capturing) return;
  capturing = which;
  const fields = { hold: 'shortcut_hold', toggle: 'shortcut_toggle', paste: 'shortcut_paste_last' };
  const field = fields[which];
  const disp = document.getElementById(which + 'Disp');
  const btn = document.getElementById(which + 'Btn');
  const prevText = disp.textContent;
  btn.textContent = 'Press shortcut...';
  btn.classList.add('cap');
  disp.textContent = '\u00a0';
  document.querySelectorAll('.btn').forEach(b => { if (b !== btn) b.style.pointerEvents = 'none'; });

  let done = false;
  const MODS = new Set(['AltRight','AltLeft','ControlRight','ControlLeft','MetaRight','MetaLeft','ShiftRight','ShiftLeft']);
  const MOD_ORD = ['ControlLeft','ControlRight','ShiftLeft','ShiftRight','AltLeft','AltRight','MetaLeft','MetaRight'];
  let held = new Set(), peak = new Set(), timer = null;

  invoke('disable_shortcuts');

  function show() {
    const p = [];
    for (const m of MOD_ORD) if (held.has(m)) p.push(keyDisp(m));
    disp.textContent = p.length ? p.join(' + ') : '\u00a0';
  }

  function onKD(e) {
    e.preventDefault(); e.stopPropagation();
    if (e.repeat || done) return;
    if (timer) { clearTimeout(timer); timer = null; }
    const c = e.code;
    if (!CODE_MAP[c]) return;
    if (MODS.has(c)) {
      held.add(c);
      if (held.size > peak.size) peak = new Set(held);
      show();
    } else {
      if (c === 'Escape' && held.size === 0) { finish(null); return; }
      const p = [];
      for (const m of MOD_ORD) if (held.has(m)) { const s = CODE_MAP[m]; if (s) p.push(s.split(':')[1]); }
      const ts = CODE_MAP[c];
      if (ts) p.push(ts.split(':')[1]);
      if (!p.length) return;
      disp.textContent = [...Array.from(held).map(m => keyDisp(m)), keyDisp(c)].join(' + ');
      finish(p.length === 1 ? CODE_MAP[c] : 'combo:' + p.join('+'));
    }
  }

  function onKU(e) {
    e.preventDefault(); e.stopPropagation();
    if (done) return;
    if (!MODS.has(e.code)) return;
    held.delete(e.code);
    if (held.size === 0) {
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => {
        const p = [];
        for (const m of MOD_ORD) if (peak.has(m)) { const s = CODE_MAP[m]; if (s) p.push(s.split(':')[1]); }
        if (!p.length) return;
        finish(p.length === 1 ? CODE_MAP[[...peak][0]] : 'combo:' + p.join('+'));
      }, 200);
      return;
    }
    show();
  }

  function finish(sc) {
    if (done) return;
    done = true;
    if (timer) { clearTimeout(timer); timer = null; }
    capturing = null;
    document.removeEventListener('keydown', onKD, true);
    document.removeEventListener('keyup', onKU, true);
    invoke('cancel_capture');
    btn.textContent = 'Set';
    btn.classList.remove('cap');
    document.querySelectorAll('.btn').forEach(b => b.style.pointerEvents = '');
    if (sc) {
      invoke('save_shortcut', { field, value: sc }).then(n => { disp.textContent = n; });
      invoke('enable_shortcuts');
    } else {
      disp.textContent = prevText;
      invoke('enable_shortcuts');
    }
  }

  document.addEventListener('keydown', onKD, true);
  document.addEventListener('keyup', onKU, true);
  invoke('capture_mouse').then(s => { if (s) finish(s); });
}

init();
listen('history-updated', () => loadHistory());
