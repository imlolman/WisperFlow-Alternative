"""WisperFlow Alternative — First-launch onboarding wizard."""

import json
import subprocess
import sys
import threading

import sounddevice as sd
import webview

from .config import CONFIG_PATH, DEFAULT_CONFIG

_CG_BTN_NAMES = {1: "right", 2: "middle", 3: "back", 4: "forward"}

SHORTCUT_DISPLAY = {
    "key:Alt_R": "Right Option ⌥", "key:Alt_L": "Left Option ⌥",
    "key:Control_R": "Right Control ⌃", "key:Control_L": "Left Control ⌃",
    "key:Super_R": "Right Cmd ⌘", "key:Super_L": "Left Cmd ⌘",
    "key:Shift_R": "Right Shift ⇧", "key:Shift_L": "Left Shift ⇧",
    "key:Caps_Lock": "Caps Lock ⇪", "key:Escape": "Escape",
    "key:space": "Space", "key:Tab": "Tab", "key:Return": "Return ↵",
    "mouse:left": "Left Click", "mouse:right": "Right Click",
    "mouse:middle": "Middle Click", "mouse:back": "Mouse Back",
    "mouse:forward": "Mouse Forward",
}
for i in range(1, 21):
    SHORTCUT_DISPLAY[f"key:F{i}"] = f"F{i}"


def _dn(s):
    if s in SHORTCUT_DISPLAY:
        return SHORTCUT_DISPLAY[s]
    if s and s.startswith("key:"):
        n = s[4:]
        return n.upper() if len(n) == 1 else n
    return s or "Not set"


class OnboardingApi:
    def __init__(self):
        self._config = DEFAULT_CONFIG.copy()
        self._cancel_ev = None

    def get_microphones(self):
        devices = sd.query_devices()
        mics = []
        for i, d in enumerate(devices):
            if d["max_input_channels"] > 0:
                mics.append({"index": i, "name": d["name"], "is_default": i == sd.default.device[0]})
        return mics

    def test_mic(self, device_index):
        """Record 0.5s from a device and return the peak amplitude."""
        import numpy as np
        try:
            buf = []
            def cb(indata, frames, t, status):
                buf.append(indata.copy())
            stream = sd.InputStream(samplerate=16000, channels=1, dtype="float32",
                                    device=int(device_index), callback=cb)
            stream.start()
            import time; time.sleep(0.5)
            stream.stop(); stream.close()
            if buf:
                audio = np.concatenate(buf).flatten()
                return float(np.max(np.abs(audio)))
            return 0.0
        except Exception as e:
            return -1

    def request_mic_permission(self):
        """Trigger mic permission prompt by briefly opening a stream."""
        try:
            stream = sd.InputStream(samplerate=16000, channels=1, dtype="float32")
            stream.start()
            import time; time.sleep(0.1)
            stream.stop(); stream.close()
            return True
        except Exception:
            return False

    def save_config(self, mic_index, hold_shortcut, toggle_shortcut):
        self._config["mic_device"] = int(mic_index) if mic_index is not None else None
        self._config["shortcut_hold"] = hold_shortcut or DEFAULT_CONFIG["shortcut_hold"]
        self._config["shortcut_toggle"] = toggle_shortcut or DEFAULT_CONFIG["shortcut_toggle"]
        self._config["setup_complete"] = True
        CONFIG_PATH.write_text(json.dumps(self._config, indent=2))
        return True

    def capture_mouse(self):
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
            tap = CGEventTapCreate(kCGSessionEventTap, kCGHeadInsertEventTap, 0x00000001, mask, cb, None)
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

    def shortcut_display(self, value):
        return _dn(value)

    def finish(self):
        global _window_ref
        if _window_ref:
            _window_ref.destroy()


HTML = r"""<!DOCTYPE html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Welcome to WisperFlow</title>
<style>
:root{--bg:#0c0c0c;--card:#161616;--border:#222;--text:#d4d4d4;
  --text2:#666;--accent:#6ba3d6;--green:#4ade80;--r:12px}
*{margin:0;padding:0;box-sizing:border-box}
html,body{background:var(--bg);color:var(--text);
  font-family:-apple-system,BlinkMacSystemFont,"SF Pro Text",system-ui,sans-serif;
  font-size:13px;line-height:1.5;-webkit-user-select:none;user-select:none;
  height:100%;overflow:hidden}
body{display:flex;flex-direction:column;padding:0}
.container{flex:1;display:flex;flex-direction:column;overflow:hidden}

.step{display:none;flex-direction:column;padding:32px 28px 24px;flex:1;overflow-y:auto}
.step.active{display:flex}

h1{font-size:22px;font-weight:700;letter-spacing:-.4px;color:#fff;margin-bottom:4px}
h2{font-size:16px;font-weight:600;color:#fff;margin-bottom:4px}
.sub{font-size:12px;color:var(--text2);margin-bottom:20px}

.card{background:var(--card);border:1px solid var(--border);
  border-radius:var(--r);padding:12px 16px;margin-bottom:12px}
.mic-item{display:flex;align-items:center;gap:10px;padding:8px 12px;
  border-radius:8px;cursor:pointer;transition:background .15s;margin:2px 0}
.mic-item:hover{background:#1c1c1c}
.mic-item.selected{background:#111828;border:1px solid #2a3a6a}
.mic-item .radio{width:16px;height:16px;border-radius:50%;border:2px solid #444;
  flex-shrink:0;display:flex;align-items:center;justify-content:center}
.mic-item.selected .radio{border-color:var(--accent)}
.mic-item.selected .radio::after{content:'';width:8px;height:8px;border-radius:50%;
  background:var(--accent)}
.mic-name{font-size:12px;flex:1}
.mic-default{font-size:9px;color:var(--text2);background:#1a1a1a;padding:2px 6px;
  border-radius:4px;text-transform:uppercase;letter-spacing:.5px}
.mic-level{height:4px;background:#1a1a1a;border-radius:2px;margin-top:8px;overflow:hidden}
.mic-level-bar{height:100%;background:var(--green);border-radius:2px;width:0;transition:width .15s}

.stitle{font-size:9px;font-weight:600;text-transform:uppercase;letter-spacing:.8px;
  color:var(--text2);margin-bottom:6px;padding-left:2px}
.srow{display:flex;align-items:center;justify-content:space-between;gap:10px;margin-bottom:8px}
.srow:last-child{margin-bottom:0}
.slabel{font-size:11px;color:var(--text2)}
.skey{font-family:"SF Mono",Menlo,monospace;font-size:12px;color:var(--accent);flex:1}
.btn-s{background:#1a1a1a;color:var(--text);border:1px solid #2a2a2a;border-radius:8px;
  padding:4px 12px;font-size:11px;font-weight:500;cursor:pointer;transition:all .15s;
  font-family:inherit}
.btn-s:hover{background:#242424;border-color:#3a3a3a}
.btn-s.cap{background:#111828;border-color:#2a3a6a;color:#7088b0;pointer-events:none;
  font-style:italic}

.footer{padding:16px 28px 24px;display:flex;justify-content:space-between;align-items:center}
.btn-next{background:var(--accent);color:#000;border:none;border-radius:10px;
  padding:8px 24px;font-size:13px;font-weight:600;cursor:pointer;transition:all .15s;
  font-family:inherit}
.btn-next:hover{filter:brightness(1.1)}
.btn-next:disabled{opacity:.4;cursor:not-allowed}
.btn-back{background:transparent;color:var(--text2);border:1px solid var(--border);
  border-radius:10px;padding:8px 16px;font-size:12px;cursor:pointer;font-family:inherit}
.btn-back:hover{border-color:#444;color:var(--text)}

.dots{display:flex;gap:6px;align-items:center}
.dot{width:6px;height:6px;border-radius:50%;background:#333;transition:background .2s}
.dot.active{background:var(--accent)}

.perm-status{display:inline-flex;align-items:center;gap:6px;font-size:11px;
  padding:4px 10px;border-radius:6px;margin-top:8px}
.perm-status.granted{background:#0a2a0a;color:var(--green);border:1px solid #1a3a1a}
.perm-status.pending{background:#2a1a0a;color:#f59e0b;border:1px solid #3a2a1a}

.done-icon{font-size:48px;text-align:center;margin:20px 0}
.done-text{text-align:center;font-size:14px;color:var(--text2);margin-bottom:16px}
</style></head><body>
<div class="container">

<!-- STEP 1: Welcome + Mic -->
<div class="step active" id="step1">
  <h1>Welcome to WisperFlow</h1>
  <p class="sub">Let's set up your microphone and shortcuts.</p>

  <div class="stitle">Select Microphone</div>
  <div class="card" id="micList" style="max-height:220px;overflow-y:auto;padding:6px 8px">
    <div style="color:#555;text-align:center;padding:12px;font-size:11px">Loading...</div>
  </div>

  <div class="mic-level" id="levelWrap" style="display:none">
    <div class="mic-level-bar" id="levelBar"></div>
  </div>

  <div id="permStatus"></div>
</div>

<!-- STEP 2: Shortcuts -->
<div class="step" id="step2">
  <h2>Configure Shortcuts</h2>
  <p class="sub">Set keys or mouse buttons to control recording.</p>

  <div class="stitle">Hold Shortcut</div>
  <div class="card">
    <div class="slabel">Press & hold to record, release to transcribe</div>
    <div class="srow" style="margin-top:8px">
      <span class="skey" id="holdDisp">Not set</span>
      <button class="btn-s" id="holdBtn" onclick="capture('hold')">Set</button>
    </div>
  </div>

  <div class="stitle">Toggle Shortcut</div>
  <div class="card">
    <div class="slabel">Press to start, press again to stop & transcribe</div>
    <div class="srow" style="margin-top:8px">
      <span class="skey" id="toggleDisp">Not set</span>
      <button class="btn-s" id="toggleBtn" onclick="capture('toggle')">Set</button>
    </div>
  </div>
</div>

<!-- STEP 3: Done -->
<div class="step" id="step3">
  <div style="flex:1;display:flex;flex-direction:column;align-items:center;justify-content:center">
    <div class="done-icon">&#10003;</div>
    <h2 style="margin-bottom:8px">You're all set!</h2>
    <p class="done-text">WisperFlow will run in your menu bar.<br>Use your shortcuts to start recording.</p>
  </div>
</div>

</div>

<div class="footer">
  <button class="btn-back" id="backBtn" onclick="prev()" style="visibility:hidden">Back</button>
  <div class="dots">
    <div class="dot active" id="d0"></div>
    <div class="dot" id="d1"></div>
    <div class="dot" id="d2"></div>
  </div>
  <button class="btn-next" id="nextBtn" onclick="next()">Continue</button>
</div>

<script>
let api,step=0,selectedMic=null,micGranted=false,levelTimer=null;
let holdShortcut=null,toggleShortcut=null,capturing=null;
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
  await loadMics();
}

async function loadMics(){
  const mics=await api.get_microphones();
  const el=document.getElementById('micList');
  el.innerHTML='';
  mics.forEach(m=>{
    const d=document.createElement('div');
    d.className='mic-item'+(m.is_default?' selected':'');
    if(m.is_default)selectedMic=m.index;
    d.innerHTML='<div class="radio"></div><span class="mic-name">'+m.name+'</span>'+
      (m.is_default?'<span class="mic-default">Default</span>':'');
    d.onclick=()=>selectMic(m.index,d);
    el.appendChild(d);
  });
  requestMicPerm();
}

function selectMic(idx,el){
  selectedMic=idx;
  document.querySelectorAll('.mic-item').forEach(e=>e.classList.remove('selected'));
  el.classList.add('selected');
  testMicLevel();
}

async function requestMicPerm(){
  const ok=await api.request_mic_permission();
  micGranted=ok;
  const s=document.getElementById('permStatus');
  if(ok){
    s.innerHTML='<span class="perm-status granted">&#10003; Microphone access granted</span>';
    testMicLevel();
  } else {
    s.innerHTML='<span class="perm-status pending">&#9888; Microphone access required — check System Settings</span>';
  }
}

async function testMicLevel(){
  if(selectedMic===null||!micGranted)return;
  const wrap=document.getElementById('levelWrap');
  const bar=document.getElementById('levelBar');
  wrap.style.display='block';
  const peak=await api.test_mic(selectedMic);
  if(peak>=0){
    const pct=Math.min(100,Math.round(peak*1000));
    bar.style.width=pct+'%';
  }
}

function capture(which){
  if(capturing)return;capturing=which;
  const disp=document.getElementById(which==='hold'?'holdDisp':'toggleDisp');
  const btn=document.getElementById(which==='hold'?'holdBtn':'toggleBtn');
  btn.textContent='Press key / click ...';btn.classList.add('cap');
  disp.textContent='Waiting...';
  let settled=false;
  function onK(e){e.preventDefault();e.stopPropagation();const s=CODE_MAP[e.code];if(!s)return;done(s)}
  function done(s){
    if(settled)return;settled=true;capturing=null;
    document.removeEventListener('keydown',onK,true);
    api.cancel_capture();
    btn.textContent='Set';btn.classList.remove('cap');
    if(which==='hold'){holdShortcut=s}else{toggleShortcut=s}
    api.shortcut_display(s).then(n=>{disp.textContent=n});
  }
  document.addEventListener('keydown',onK,true);
  api.capture_mouse().then(s=>{if(s)done(s)});
}

function showStep(n){
  document.querySelectorAll('.step').forEach(s=>s.classList.remove('active'));
  document.getElementById('step'+(n+1)).classList.add('active');
  for(let i=0;i<3;i++)document.getElementById('d'+i).classList.toggle('active',i===n);
  document.getElementById('backBtn').style.visibility=n===0?'hidden':'visible';
  const nb=document.getElementById('nextBtn');
  if(n===2){nb.textContent='Start App';nb.disabled=false}
  else{nb.textContent='Continue';nb.disabled=false}
  step=n;
}

async function next(){
  if(step===0){showStep(1)}
  else if(step===1){showStep(2)}
  else{
    document.getElementById('nextBtn').disabled=true;
    document.getElementById('nextBtn').textContent='Starting...';
    await api.save_config(selectedMic,holdShortcut,toggleShortcut);
    await api.finish();
  }
}
function prev(){if(step>0)showStep(step-1)}

window.addEventListener('pywebviewready',init);
</script></body></html>"""


_window_ref = None


def run_onboarding() -> bool:
    """Show onboarding wizard. Returns True if setup was completed."""
    global _window_ref
    api = OnboardingApi()
    _window_ref = webview.create_window(
        "Welcome to WisperFlow", html=HTML, js_api=api,
        width=420, height=520, resizable=False, background_color="#0c0c0c",
    )
    webview.start()
    try:
        cfg = json.loads(CONFIG_PATH.read_text())
        return cfg.get("setup_complete", False)
    except Exception:
        return False
