"""WisperFlow Alternative — Settings UI (pywebview)."""

import json
import plistlib
import subprocess
import sys
import threading
from pathlib import Path

import sounddevice as sd
import webview

from .config import CONFIG_PATH, HISTORY_PATH, DEFAULT_CONFIG

LAUNCH_AGENT_PATH = Path.home() / "Library/LaunchAgents/com.wisper.app.plist"
APP_SCRIPT = Path(__file__).resolve().parent / "app.py"

SHORTCUT_DISPLAY = {
    "key:Alt_R": "Right Option ⌥", "key:Alt_L": "Left Option ⌥",
    "key:Control_R": "Right Control ⌃", "key:Control_L": "Left Control ⌃",
    "key:Super_R": "Right Cmd ⌘", "key:Super_L": "Left Cmd ⌘",
    "key:Meta_R": "Right Cmd ⌘", "key:Meta_L": "Left Cmd ⌘",
    "key:Shift_R": "Right Shift ⇧", "key:Shift_L": "Left Shift ⇧",
    "key:Caps_Lock": "Caps Lock ⇪", "key:Escape": "Escape",
    "key:space": "Space", "key:Tab": "Tab", "key:Return": "Return ↵",
    "key:BackSpace": "Backspace ⌫", "key:Delete": "Delete",
    "mouse:left": "Left Click 🖱",
    "mouse:right": "Right Click 🖱",
    "mouse:middle": "Middle Click 🖱",
    "mouse:back": "Mouse Back 🖱",
    "mouse:forward": "Mouse Fwd 🖱",
}
for i in range(1, 21):
    SHORTCUT_DISPLAY[f"key:F{i}"] = f"F{i}"


def _dn(s):
    if s in SHORTCUT_DISPLAY:
        return SHORTCUT_DISPLAY[s]
    if s and s.startswith("key:"):
        n = s[4:]
        return n.upper() if len(n) == 1 else n
    return s or "None"


def _load():
    try:
        if CONFIG_PATH.exists():
            c = json.loads(CONFIG_PATH.read_text())
            for old in ("shortcut", "shortcut_mode", "shortcut_key", "hotkey", "model"):
                c.pop(old, None)
            for k, v in DEFAULT_CONFIG.items():
                c.setdefault(k, v)
            return c
    except Exception:
        pass
    return DEFAULT_CONFIG.copy()


def _save(c):
    CONFIG_PATH.write_text(json.dumps(c, indent=2))


def _install_la():
    p = {"Label": "com.wisper.app", "ProgramArguments": [sys.executable, str(APP_SCRIPT)],
         "RunAtLoad": True, "KeepAlive": True, "WorkingDirectory": str(APP_SCRIPT.parent),
         "EnvironmentVariables": {"PATH": "/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"}}
    LAUNCH_AGENT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with open(LAUNCH_AGENT_PATH, "wb") as f:
        plistlib.dump(p, f)


def _remove_la():
    if LAUNCH_AGENT_PATH.exists():
        LAUNCH_AGENT_PATH.unlink()


_CG_BTN_NAMES = {1: "right", 2: "middle", 3: "back", 4: "forward"}


class Api:
    _cancel_ev = None

    def get_config(self):
        c = _load()
        c["_hold_display"] = _dn(c["shortcut_hold"])
        c["_toggle_display"] = _dn(c["shortcut_toggle"])
        return c

    def get_microphones(self):
        devices = sd.query_devices()
        mics = []
        cfg = _load()
        selected = cfg.get("mic_device")
        for i, d in enumerate(devices):
            if d["max_input_channels"] > 0:
                mics.append({
                    "index": i, "name": d["name"],
                    "is_default": i == sd.default.device[0],
                    "is_selected": (selected == i) if selected is not None else (i == sd.default.device[0]),
                })
        return mics

    def save_mic(self, device_index):
        c = _load()
        c["mic_device"] = int(device_index) if device_index is not None else None
        _save(c)
        return True

    def save_shortcut(self, field, value):
        c = _load()
        c[field] = value
        _save(c)
        return _dn(value)

    def save_field(self, field, value):
        c = _load()
        c[field] = value
        _save(c)
        if field == "start_on_login":
            (_install_la if value else _remove_la)()
        return True

    def get_history(self):
        try:
            if HISTORY_PATH.exists():
                return json.loads(HISTORY_PATH.read_text())
        except Exception:
            pass
        return []

    def copy_text(self, text):
        subprocess.run(["pbcopy"], input=text.encode("utf-8"), check=True)

    def capture_mouse(self):
        """Wait for a non-left mouse click using CGEvent tap. Returns 'mouse:xxx' or None."""
        from Quartz import (
            CGEventGetIntegerValueField, CGEventMaskBit,
            CGEventTapCreate, CGEventTapEnable,
            CFMachPortCreateRunLoopSource, CFRunLoopAddSource,
            CFRunLoopGetCurrent, CFRunLoopRun, CFRunLoopStop,
            kCFRunLoopCommonModes,
            kCGEventRightMouseDown, kCGEventOtherMouseDown,
            kCGMouseEventButtonNumber, kCGSessionEventTap, kCGHeadInsertEventTap,
        )

        result = [None]
        done = threading.Event()
        rl_ref = [None]
        cancel = threading.Event()
        self._cancel_ev = cancel

        def _tap_thread():
            mask = CGEventMaskBit(kCGEventRightMouseDown) | CGEventMaskBit(kCGEventOtherMouseDown)

            def cb(proxy, etype, ev, refcon):
                if etype == kCGEventRightMouseDown:
                    num = 1
                else:
                    num = CGEventGetIntegerValueField(ev, kCGMouseEventButtonNumber)
                name = _CG_BTN_NAMES.get(num, str(num))
                result[0] = f"mouse:{name}"
                done.set()
                if rl_ref[0]:
                    CFRunLoopStop(rl_ref[0])
                return ev

            tap = CGEventTapCreate(
                kCGSessionEventTap, kCGHeadInsertEventTap,
                0x00000001, mask, cb, None,
            )
            if tap is None:
                done.set()
                return
            src = CFMachPortCreateRunLoopSource(None, tap, 0)
            rl_ref[0] = CFRunLoopGetCurrent()
            CFRunLoopAddSource(rl_ref[0], src, kCFRunLoopCommonModes)
            CGEventTapEnable(tap, True)
            CFRunLoopRun()

        t = threading.Thread(target=_tap_thread, daemon=True)
        t.start()

        while not done.wait(timeout=0.2):
            if cancel.is_set():
                break

        if rl_ref[0]:
            CFRunLoopStop(rl_ref[0])
        self._cancel_ev = None
        return result[0]

    def cancel_capture(self):
        ev = self._cancel_ev
        if ev:
            ev.set()


HTML = r"""<!DOCTYPE html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>WisperFlow Alternative</title>
<style>
:root{--bg:#0c0c0c;--card:#161616;--border:#222;--text:#d4d4d4;
  --text2:#555;--accent:#6ba3d6;--r:12px}
*{margin:0;padding:0;box-sizing:border-box}
html,body{background:var(--bg);color:var(--text);
  font-family:-apple-system,BlinkMacSystemFont,"SF Pro Text",system-ui,sans-serif;
  font-size:13px;line-height:1.5;-webkit-user-select:none;user-select:none}
body{padding:28px 24px 20px}
h1{font-size:18px;font-weight:700;letter-spacing:-.3px;margin-bottom:22px;color:#fff}
.section{margin-bottom:14px}
.stitle{font-size:9px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;
  color:var(--text2);margin-bottom:5px;padding-left:2px}
.card{background:var(--card);border:1px solid var(--border);
  border-radius:var(--r);padding:10px 14px}
.row{display:flex;align-items:center;justify-content:space-between;gap:10px}
.skey{font-family:"SF Mono",Menlo,monospace;font-size:12px;color:var(--accent);flex:1}
.btn{background:#1a1a1a;color:var(--text);border:1px solid #2a2a2a;border-radius:8px;
  padding:4px 12px;font-size:11px;font-weight:500;cursor:pointer;transition:all .15s;
  font-family:inherit}
.btn:hover{background:#242424;border-color:#3a3a3a}
.btn.cap{background:#111828;border-color:#2a3a6a;color:#7088b0;pointer-events:none;
  font-style:italic}
.lbl{font-size:10px;color:var(--text2);margin-bottom:5px}
.sep{height:1px;background:var(--border);margin:8px 0}
.check-row{display:flex;align-items:center;gap:8px;padding:3px 0;cursor:pointer}
.check-row input{accent-color:#555;width:13px;height:13px;cursor:pointer}
.check-row label{cursor:pointer;font-size:11px;color:var(--text)}
.hlist{max-height:180px;overflow-y:auto}
.hlist::-webkit-scrollbar{width:3px}
.hlist::-webkit-scrollbar-thumb{background:#252525;border-radius:2px}
.hi{display:flex;align-items:center;padding:6px 12px;border-bottom:1px solid #1a1a1a;gap:8px}
.hi:hover{background:#1a1a1a}.hi:last-child{border:none}
.ht{font-size:10px;color:#444;font-family:"SF Mono",monospace;min-width:36px}
.hx{flex:1;font-size:11px;color:#999;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.hc{background:transparent;border:1px solid #2a2a2a;border-radius:6px;color:#555;
  font-size:9px;padding:2px 8px;cursor:pointer;transition:all .15s;font-family:inherit}
.hc:hover{border-color:#444;color:#ccc}
.empty{color:#444;text-align:center;padding:20px;font-size:11px}
</style></head><body>
<h1>WisperFlow Alternative</h1>

<div class="section">
  <div class="stitle">Microphone</div>
  <div class="card">
    <select id="micSel" onchange="saveMic()" style="width:100%;background:#1a1a1a;color:var(--text);
      border:1px solid #2a2a2a;border-radius:8px;padding:6px 10px;font-size:11px;
      font-family:inherit;cursor:pointer;-webkit-appearance:none;appearance:none">
      <option value="">Loading...</option>
    </select>
  </div>
</div>

<div class="section">
  <div class="stitle">Hold Shortcut</div>
  <div class="card">
    <div class="lbl">Press &amp; hold to record, release to transcribe</div>
    <div class="row">
      <span class="skey" id="holdDisp">...</span>
      <button class="btn" id="holdBtn" onclick="capture('hold')">Set</button>
    </div>
  </div>
</div>

<div class="section">
  <div class="stitle">Toggle Shortcut</div>
  <div class="card">
    <div class="lbl">Press to start, press again to stop &amp; transcribe</div>
    <div class="row">
      <span class="skey" id="toggleDisp">...</span>
      <button class="btn" id="toggleBtn" onclick="capture('toggle')">Set</button>
    </div>
  </div>
</div>

<div class="section">
  <div class="card">
    <div class="check-row" onclick="tog('hideTray')">
      <input type="checkbox" id="hideTray"><label>Auto-hide tray icon</label>
    </div>
    <div class="sep"></div>
    <div class="check-row" onclick="tog('startLogin')">
      <input type="checkbox" id="startLogin"><label>Start on login</label>
    </div>
  </div>
</div>

<div class="section" style="flex:1">
  <div class="stitle">History</div>
  <div class="card" style="padding:4px 0"><div class="hlist" id="hl"><div class="empty">...</div></div></div>
</div>

<script>
let api,capturing=null;
const CODE_MAP={AltRight:'key:Alt_R',AltLeft:'key:Alt_L',
  ControlRight:'key:Control_R',ControlLeft:'key:Control_L',
  MetaRight:'key:Super_R',MetaLeft:'key:Super_L',
  ShiftRight:'key:Shift_R',ShiftLeft:'key:Shift_L',
  CapsLock:'key:Caps_Lock',Escape:'key:Escape',
  Space:'key:space',Tab:'key:Tab',Backspace:'key:BackSpace',
  Enter:'key:Return',Delete:'key:Delete',Home:'key:Home',End:'key:End'};
for(let i=1;i<=20;i++)CODE_MAP['F'+i]='key:F'+i;
for(let i=0;i<=9;i++)CODE_MAP['Digit'+i]='key:'+i;
for(let c=65;c<=90;c++){const ch=String.fromCharCode(c);CODE_MAP['Key'+ch]='key:'+ch.toLowerCase()}
async function init(){
  api=window.pywebview.api;
  const c=await api.get_config();
  document.getElementById('holdDisp').textContent=c._hold_display;
  document.getElementById('toggleDisp').textContent=c._toggle_display;
  document.getElementById('hideTray').checked=c.hide_tray;
  document.getElementById('startLogin').checked=c.start_on_login;
  loadMics();
  loadH();
}
async function loadMics(){
  const mics=await api.get_microphones();
  const sel=document.getElementById('micSel');
  sel.innerHTML='';
  mics.forEach(m=>{
    const o=document.createElement('option');
    o.value=m.index;o.textContent=m.name+(m.is_default?' (Default)':'');
    if(m.is_selected)o.selected=true;
    sel.appendChild(o);
  });
}
function saveMic(){
  const v=document.getElementById('micSel').value;
  api.save_mic(v===''?null:parseInt(v));
}

function capture(which){
  if(capturing)return;capturing=which;
  const field=which==='hold'?'shortcut_hold':'shortcut_toggle';
  const disp=document.getElementById(which==='hold'?'holdDisp':'toggleDisp');
  const btn=document.getElementById(which==='hold'?'holdBtn':'toggleBtn');
  btn.textContent='Press key / click ...';btn.classList.add('cap');
  disp.textContent='Waiting...';
  let settled=false;
  function onK(e){e.preventDefault();e.stopPropagation();const s=CODE_MAP[e.code];if(!s)return;done(s)}
  function done(s){
    if(settled)return;settled=true;
    capturing=null;
    document.removeEventListener('keydown',onK,true);
    api.cancel_capture();
    btn.textContent='Set';btn.classList.remove('cap');
    api.save_shortcut(field,s).then(n=>{disp.textContent=n});
  }
  document.addEventListener('keydown',onK,true);
  api.capture_mouse().then(s=>{if(s)done(s)});
}

function tog(id){
  const el=document.getElementById(id);el.checked=!el.checked;
  if(id==='hideTray')api.save_field('hide_tray',el.checked);
  if(id==='startLogin')api.save_field('start_on_login',el.checked);
}

async function loadH(){
  const l=document.getElementById('hl');
  const h=await api.get_history();
  if(!h.length){l.innerHTML='<div class="empty">No transcriptions yet</div>';return}
  l.innerHTML='';
  h.slice().reverse().forEach(e=>{
    const d=document.createElement('div');d.className='hi';
    const ts=e.timestamp?e.timestamp.substring(11,16):'';
    const t=e.text||'';
    const esc=t.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/"/g,'&quot;');
    const esc2=t.replace(/\\/g,'\\\\').replace(/'/g,"\\'");
    d.innerHTML='<span class="ht">'+ts+'</span><span class="hx" title="'+esc+'">'+esc+
      '</span><button class="hc" onclick="cp(this,\''+esc2+'\')">Copy</button>';
    l.appendChild(d);
  });
}

async function cp(b,t){await api.copy_text(t);b.textContent='\u2713';setTimeout(()=>b.textContent='Copy',1000)}

window.addEventListener('pywebviewready',init);
</script></body></html>"""


def run_settings():
    """Entry point for in-process or --settings launch (used by frozen app)."""
    window = webview.create_window(
        "WisperFlow Alternative", html=HTML, js_api=Api(),
        width=380, height=500, resizable=False, background_color="#0c0c0c",
    )
    webview.start()


if __name__ == "__main__":
    run_settings()
