const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let step = 0;
let selectedMic = null;
let holdShortcut = 'mouse:middle';
let toggleShortcut = 'key:Alt_R';
let capturing = null;
let modelReady = false;
let platform = 'macos';
let permissionPollInterval = null;
let allPermissionsGranted = false;

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
'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('').forEach(c => { CODE_MAP['Key' + c] = 'key:' + c.toLowerCase(); });

function keyDisp(code) {
  if (platform === 'macos') {
    const m = {
      ControlLeft:'\u2303', ControlRight:'\u2303',
      ShiftLeft:'\u21e7', ShiftRight:'\u21e7',
      AltLeft:'\u2325', AltRight:'\u2325',
      MetaLeft:'\u2318', MetaRight:'\u2318',
      Escape:'Esc', Enter:'\u21a9', Backspace:'\u232b',
      Delete:'\u2326', Tab:'\u21e5', Space:'\u2423',
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
  return code;
}

async function init() {
  platform = await invoke('get_platform');
  updatePlatformLabels();
}

function showStep(n) {
  document.querySelectorAll('.step').forEach(s => s.classList.remove('active'));
  document.getElementById('step' + (n + 1)).classList.add('active');
  for (let i = 0; i < 5; i++) {
    document.getElementById('d' + i).classList.toggle('on', i === n);
  }
  document.getElementById('backBtn').style.visibility = n === 0 ? 'hidden' : 'visible';
  if (n === 4) {
    document.getElementById('nextBtn').textContent = 'Start App';
  } else {
    document.getElementById('nextBtn').textContent = 'Continue';
  }

  // Handle step-specific logic
  if (n === 1) {
    renderPermissionCards();
    startPermissionPolling();
  } else {
    stopPermissionPolling();
  }

  if (n === 2) {
    loadMicrophones();
  }
}

async function next() {
  if (step === 0) {
    step = 1;
    showStep(1);
  } else if (step === 1) {
    if (!allPermissionsGranted) return;
    step = 2;
    showStep(2);
  } else if (step === 2) {
    if (!modelReady) return;
    step = 3;
    showStep(3);
  } else if (step === 3) {
    step = 4;
    showStep(4);
  } else if (step === 4) {
    invoke('finish_onboarding', { mic: selectedMic, hold: holdShortcut, toggle: toggleShortcut });
  }
}

function back() {
  if (step > 0) {
    step--;
    showStep(step);
  }
}

// ===== PERMISSIONS STEP =====

function renderPermissionCards() {
  const container = document.getElementById('permissionCards');
  container.innerHTML = '';

  if (platform === 'macos') {
    container.innerHTML = `
      <div class="perm-card" id="micPermCard">
        <div class="perm-header">
          <div class="perm-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
              <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
              <line x1="12" y1="19" x2="12" y2="23"/>
              <line x1="8" y1="23" x2="16" y2="23"/>
            </svg>
          </div>
          <div class="perm-info">
            <div class="perm-title">Microphone</div>
            <div class="perm-desc">Required to capture your voice for transcription</div>
          </div>
        </div>
        <div class="perm-footer">
          <div class="perm-status" id="micStatus">
            <span class="status-icon status-pending">⏳</span>
            <span>Checking...</span>
          </div>
          <button class="perm-action" id="micGrantBtn" onclick="requestMicPermission()">Grant Access</button>
        </div>
      </div>
      <div class="perm-card" id="accessibilityPermCard">
        <div class="perm-header">
          <div class="perm-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/>
              <path d="M8 14s1.5 2 4 2 4-2 4-2"/>
              <line x1="9" y1="9" x2="9.01" y2="9"/>
              <line x1="15" y1="9" x2="15.01" y2="9"/>
            </svg>
          </div>
          <div class="perm-info">
            <div class="perm-title">Accessibility</div>
            <div class="perm-desc">Required for global shortcuts and typing into other apps</div>
          </div>
        </div>
        <div class="perm-footer">
          <div class="perm-status" id="accessibilityStatus">
            <span class="status-icon status-pending">⏳</span>
            <span>Checking...</span>
          </div>
          <button class="perm-action" id="accessibilityGrantBtn" onclick="openAccessibilitySettings()">Open Settings</button>
        </div>
      </div>
    `;
  } else if (platform === 'windows') {
    container.innerHTML = `
      <div class="perm-card" id="micPermCard">
        <div class="perm-header">
          <div class="perm-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
              <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
              <line x1="12" y1="19" x2="12" y2="23"/>
              <line x1="8" y1="23" x2="16" y2="23"/>
            </svg>
          </div>
          <div class="perm-info">
            <div class="perm-title">Microphone</div>
            <div class="perm-desc">Required to capture your voice for transcription</div>
          </div>
        </div>
        <div class="perm-footer">
          <div class="perm-status" id="micStatus">
            <span class="status-icon status-pending">⏳</span>
            <span>Checking...</span>
          </div>
          <button class="perm-action" id="micGrantBtn" onclick="requestMicPermission()">Grant Access</button>
        </div>
      </div>
    `;
  } else {
    container.innerHTML = `
      <div class="perm-card" id="micPermCard">
        <div class="perm-header">
          <div class="perm-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3z"/>
              <path d="M19 10v2a7 7 0 0 1-14 0v-2"/>
              <line x1="12" y1="19" x2="12" y2="23"/>
              <line x1="8" y1="23" x2="16" y2="23"/>
            </svg>
          </div>
          <div class="perm-info">
            <div class="perm-title">Microphone</div>
            <div class="perm-desc">Required to capture your voice for transcription</div>
          </div>
        </div>
        <div class="perm-footer">
          <div class="perm-status" id="micStatus">
            <span class="status-icon status-pending">⏳</span>
            <span>Checking...</span>
          </div>
          <button class="perm-action" id="micGrantBtn" onclick="requestMicPermission()">Grant Access</button>
        </div>
      </div>
      <div class="perm-card" id="inputPermCard">
        <div class="perm-header">
          <div class="perm-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="4" width="20" height="16" rx="2"/>
              <path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h8"/>
            </svg>
          </div>
          <div class="perm-info">
            <div class="perm-title">Input Access</div>
            <div class="perm-desc">Required for global shortcuts to work</div>
          </div>
        </div>
        <div class="perm-footer">
          <div class="perm-status" id="inputStatus">
            <span class="status-icon status-pending">⏳</span>
            <span>Checking...</span>
          </div>
        </div>
        <div class="linux-instructions">
          Run this command in terminal, then restart the app:<br>
          <code>sudo usermod -a -G input $USER</code>
        </div>
      </div>
    `;
  }
}

async function checkPermissions() {
  try {
    const status = await invoke('check_permissions');
    
    // Update microphone status
    const micStatus = document.getElementById('micStatus');
    const micGrantBtn = document.getElementById('micGrantBtn');
    if (status.microphone) {
      micStatus.innerHTML = '<span class="status-icon status-granted">✓</span><span>Granted</span>';
      micStatus.className = 'perm-status status-granted';
      if (micGrantBtn) micGrantBtn.disabled = true;
    } else {
      micStatus.innerHTML = '<span class="status-icon status-pending">⏳</span><span>Pending</span>';
      micStatus.className = 'perm-status status-pending';
      if (micGrantBtn) micGrantBtn.disabled = false;
    }

    // Update accessibility status (macOS only)
    if (platform === 'macos') {
      const accessibilityStatus = document.getElementById('accessibilityStatus');
      const accessibilityGrantBtn = document.getElementById('accessibilityGrantBtn');
      if (status.accessibility) {
        accessibilityStatus.innerHTML = '<span class="status-icon status-granted">✓</span><span>Granted</span>';
        accessibilityStatus.className = 'perm-status status-granted';
        if (accessibilityGrantBtn) accessibilityGrantBtn.disabled = true;
      } else {
        accessibilityStatus.innerHTML = '<span class="status-icon status-pending">⏳</span><span>Pending</span>';
        accessibilityStatus.className = 'perm-status status-pending';
        if (accessibilityGrantBtn) accessibilityGrantBtn.disabled = false;
      }
    }

    // Update input access status (Linux only)
    if (platform === 'linux') {
      const inputStatus = document.getElementById('inputStatus');
      if (status.input_access) {
        inputStatus.innerHTML = '<span class="status-icon status-granted">✓</span><span>Granted</span>';
        inputStatus.className = 'perm-status status-granted';
      } else {
        inputStatus.innerHTML = '<span class="status-icon status-pending">⏳</span><span>Pending</span>';
        inputStatus.className = 'perm-status status-pending';
      }
    }

    // Check if all required permissions are granted
    if (platform === 'macos') {
      allPermissionsGranted = status.microphone && status.accessibility;
    } else if (platform === 'linux') {
      allPermissionsGranted = status.microphone && status.input_access;
    } else {
      allPermissionsGranted = status.microphone;
    }

    // Enable/disable continue button
    const nextBtn = document.getElementById('nextBtn');
    if (step === 1) {
      nextBtn.disabled = !allPermissionsGranted;
    }
  } catch (e) {
    console.error('Failed to check permissions:', e);
  }
}

function startPermissionPolling() {
  checkPermissions();
  permissionPollInterval = setInterval(checkPermissions, 2000);
}

function stopPermissionPolling() {
  if (permissionPollInterval) {
    clearInterval(permissionPollInterval);
    permissionPollInterval = null;
  }
}

async function requestMicPermission() {
  try {
    await invoke('request_mic_permission');
    setTimeout(checkPermissions, 500);
  } catch (e) {
    console.error('Failed to request mic permission:', e);
  }
}

async function openAccessibilitySettings() {
  try {
    await invoke('open_accessibility_settings');
  } catch (e) {
    console.error('Failed to open accessibility settings:', e);
  }
}

// ===== MIC + MODEL STEP =====

async function loadMicrophones() {
  const mics = await invoke('get_microphones');
  const list = document.getElementById('micList');
  list.innerHTML = '';
  mics.forEach((m, i) => {
    const div = document.createElement('div');
    div.className = 'mic-item' + (m.is_default ? ' sel' : '');
    div.innerHTML = `<div class="mic-radio"></div><span>${m.name}${m.is_default ? ' (Default)' : ''}</span>`;
    div.onclick = () => {
      list.querySelectorAll('.mic-item').forEach(el => el.classList.remove('sel'));
      div.classList.add('sel');
      selectedMic = m.name;
      testMic();
    };
    if (m.is_default) selectedMic = m.name;
    list.appendChild(div);
  });

  checkAndDownloadModel();
}

async function testMic() {
  document.getElementById('levelWrap').style.display = 'block';
  try {
    const peak = await invoke('test_mic', { device: selectedMic });
    const pct = Math.min(100, Math.round(peak * 100 * 3));
    document.getElementById('levelFill').style.width = pct + '%';
  } catch (e) {
    document.getElementById('levelFill').style.width = '0%';
  }
}

async function checkAndDownloadModel() {
  const exists = await invoke('check_model_exists');
  if (exists) {
    modelReady = true;
    return;
  }
  document.getElementById('dlProgress').style.display = 'block';
  document.getElementById('nextBtn').disabled = true;

  listen('model-download-progress', (event) => {
    const [downloaded, total] = event.payload;
    if (total > 0) {
      const pct = Math.round((downloaded / total) * 100);
      document.getElementById('dlFill').style.width = pct + '%';
      document.getElementById('dlText').textContent = `Downloading speech model... ${pct}%`;
    }
  });

  try {
    await invoke('download_model');
    document.getElementById('dlText').textContent = 'Model downloaded!';
    document.getElementById('dlFill').style.width = '100%';
    modelReady = true;
    document.getElementById('nextBtn').disabled = false;
  } catch (e) {
    document.getElementById('dlText').textContent = 'Download failed: ' + e;
  }
}

// ===== SHORTCUTS STEP =====

function capture(which) {
  if (capturing) return;
  capturing = which;
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
      if (which === 'hold') holdShortcut = sc;
      else toggleShortcut = sc;
      invoke('shortcut_display_name', { value: sc }).then(n => { disp.textContent = n; });
    } else {
      disp.textContent = prevText;
    }
  }

  document.addEventListener('keydown', onKD, true);
  document.addEventListener('keyup', onKU, true);
  invoke('capture_mouse').then(s => { if (s) finish(s); });
}

function updatePlatformLabels() {
  if (platform !== 'macos') {
    const toggleDisp = document.getElementById('toggleDisp');
    const doneSubtext = document.getElementById('doneSubtext');
    if (toggleDisp) toggleDisp.textContent = 'Right Alt';
    if (doneSubtext) doneSubtext.innerHTML = 'OpenBolo runs in your system tray.<br>Use your shortcuts to start dictating.';
  }
}

init();
