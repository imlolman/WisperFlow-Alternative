"""WisperFlow Alternative — cross-platform tray application."""

import os
import subprocess
import sys
import threading
import time
import tkinter as tk
from pathlib import Path

import numpy as np
import pystray
import sounddevice as sd
from PIL import Image, ImageDraw

from .clipboard import copy_to_clipboard, simulate_paste
from .config import CONFIG_PATH, SAMPLE_RATE, load_config, append_history
from .ipc import start_command_server
from .overlay import OverlayWindow
from .shortcuts import RawKeyboardListener, RawMouseListener, parse_shortcut, keys_match
from .transcriber import get_device, load_model, transcribe

_FROZEN = getattr(sys, "frozen", False)
_PROJECT_ROOT = (
    Path(sys.executable).resolve().parent if _FROZEN
    else Path(__file__).resolve().parent.parent
)


def _make_icon(color: tuple) -> Image.Image:
    size = 64
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.ellipse([6, 6, size - 6, size - 6], fill=color)
    return img


_ICON_IDLE = _make_icon((70, 70, 70, 230))
_ICON_LOADING = _make_icon((60, 100, 160, 230))
_ICON_RECORDING = _make_icon((200, 50, 50, 230))
_ICON_PROCESSING = _make_icon((200, 160, 30, 230))


class WFApp:
    def __init__(self):
        self.config = load_config()
        self.is_recording = False
        self.is_processing = False
        self._recording_mode: str | None = None
        self.audio_buffer: list[np.ndarray] = []
        self.stream: sd.InputStream | None = None
        self.overlay: OverlayWindow | None = None
        self.whisper_model = None
        self.device: str | None = None
        self._key_listener = None
        self._mouse_listener = None
        self._last_hold_end: float = 0.0
        self._config_mtime: float = 0
        self._settings_proc: subprocess.Popen | None = None
        self._latest_rms: float = 0.0
        self._amp_after_id = None
        self._tk_root: tk.Tk | None = None
        self._icon: pystray.Icon | None = None

    def _call_after(self, fn):
        if self._tk_root:
            self._tk_root.after(0, fn)

    def run(self):
        self._tk_root = tk.Tk()
        self._tk_root.withdraw()
        self._tk_root.protocol("WM_DELETE_WINDOW", lambda: None)

        self.overlay = OverlayWindow(self._tk_root, on_cancel=self._cancel_recording)

        menu = pystray.Menu(
            pystray.MenuItem("Settings", lambda *_: self._call_after(self.open_settings)),
            pystray.Menu.SEPARATOR,
            pystray.MenuItem("Quit", lambda *_: self._call_after(self._quit)),
        )
        self._icon = pystray.Icon("WisperFlow", _ICON_LOADING, "WisperFlow (loading…)", menu)
        self._icon.run_detached()

        threading.Thread(
            target=lambda: start_command_server(self._call_after, self), daemon=True
        ).start()
        threading.Thread(target=self._load_model, daemon=True).start()
        threading.Thread(target=self._watch_config, daemon=True).start()

        try:
            self._tk_root.mainloop()
        finally:
            if self._icon:
                self._icon.stop()
            self._stop_all_listeners()

    def _quit(self):
        self._stop_all_listeners()
        if self._icon:
            self._icon.stop()
        if self._tk_root:
            self._tk_root.quit()

    # ── Tray visibility (no-op stubs kept for ipc.py compatibility) ──

    def _show_tray_temporarily(self):
        pass

    # ── Settings ─────────────────────────────────────────────────────

    def open_settings(self):
        if self._settings_proc and self._settings_proc.poll() is None:
            return
        if _FROZEN:
            self._settings_proc = subprocess.Popen(
                [sys.executable, "--settings"],
                cwd=os.path.expanduser("~"),
                env=os.environ,
            )
        else:
            self._settings_proc = subprocess.Popen(
                [sys.executable, "-m", "wisperflow.ui"],
                cwd=str(_PROJECT_ROOT),
            )

    # ── Config watcher ───────────────────────────────────────────────

    def _watch_config(self):
        while True:
            try:
                if CONFIG_PATH.exists():
                    mt = CONFIG_PATH.stat().st_mtime
                    if mt != self._config_mtime:
                        self._config_mtime = mt
                        new = load_config()
                        old = self.config
                        self.config = new
                        if (new["shortcut_hold"] != old.get("shortcut_hold")
                                or new["shortcut_toggle"] != old.get("shortcut_toggle")):
                            self._restart_listeners()
                            print(f"[wf] Shortcuts: hold={new['shortcut_hold']}  toggle={new['shortcut_toggle']}")
            except Exception:
                pass
            time.sleep(2)

    # ── Model ────────────────────────────────────────────────────────

    def _load_model(self):
        self.device = get_device()
        print(f"[wf] Loading base.en on {self.device} ...")
        t0 = time.time()
        self.whisper_model = load_model(self.device)
        print(f"[wf] Model ready ({time.time() - t0:.1f}s)")
        if self._icon:
            self._icon.icon = _ICON_IDLE
            self._icon.title = "WisperFlow"
        self._restart_listeners()

    # ── Listeners ────────────────────────────────────────────────────

    def _stop_all_listeners(self):
        for attr in ("_key_listener", "_mouse_listener"):
            lst = getattr(self, attr, None)
            if lst:
                lst.stop()
                setattr(self, attr, None)

    def _notify_accessibility_fail(self):
        if self._icon:
            self._icon.notify(
                "Accessibility permission required",
                "Grant access in System Settings > Privacy > Accessibility, then restart.",
            )

    def _restart_listeners(self):
        self._stop_all_listeners()

        hold_kind, hold_target = parse_shortcut(self.config["shortcut_hold"])
        toggle_kind, toggle_target = parse_shortcut(self.config["shortcut_toggle"])

        has_key_hold = hold_kind == "key" and hold_target is not None
        has_key_toggle = toggle_kind == "key" and toggle_target is not None
        has_mouse_hold = hold_kind == "mouse" and hold_target is not None
        has_mouse_toggle = toggle_kind == "mouse" and toggle_target is not None

        def hold_press():
            if self.whisper_model is None:
                return
            if self.is_recording and self._recording_mode == "toggle":
                self._stop_recording()
                return
            if self.is_recording or self.is_processing:
                return
            if time.time() - self._last_hold_end < 1.0:
                self._start_recording("toggle")
            else:
                self._start_recording("hold")

        def hold_release():
            if self.is_recording and self._recording_mode == "hold":
                self._last_hold_end = time.time()
                self._stop_recording()

        def toggle_press():
            if self.whisper_model is None:
                return
            if self.is_recording and self._recording_mode == "toggle":
                self._stop_recording()
            elif not self.is_recording and not self.is_processing:
                self._start_recording("toggle")

        if has_key_hold or has_key_toggle:
            key_hold = hold_target if has_key_hold else None
            key_toggle = toggle_target if has_key_toggle else None

            def on_key_press(key):
                if key_hold is not None and keys_match(key, key_hold):
                    hold_press()
                elif key_toggle is not None and keys_match(key, key_toggle):
                    toggle_press()

            def on_key_release(key):
                if key_hold is not None and keys_match(key, key_hold):
                    hold_release()

            self._key_listener = RawKeyboardListener(
                on_key_press, on_key_release,
                on_tap_failed=self._notify_accessibility_fail,
            )
            self._key_listener.start()

        if has_mouse_hold or has_mouse_toggle:
            mt_hold = hold_target if has_mouse_hold else None
            mt_toggle = toggle_target if has_mouse_toggle else None

            def on_mouse(btn_name, pressed):
                if mt_hold is not None and btn_name == mt_hold:
                    if pressed:
                        hold_press()
                    else:
                        hold_release()
                elif mt_toggle is not None and btn_name == mt_toggle and pressed:
                    toggle_press()

            self._mouse_listener = RawMouseListener(
                on_mouse,
                on_tap_failed=self._notify_accessibility_fail,
            )
            self._mouse_listener.start()

        print(f"[wf] Listening: hold={self.config['shortcut_hold']}  toggle={self.config['shortcut_toggle']}")

    # ── Recording ────────────────────────────────────────────────────

    def _start_recording(self, mode: str):
        self.is_recording = True
        self._recording_mode = mode
        self.audio_buffer = []
        self._latest_rms = 0.0

        def _audio_cb(indata, frames, time_info, status):
            self.audio_buffer.append(indata.copy())
            self._latest_rms = float(np.sqrt(np.mean(indata ** 2)))

        self.stream = sd.InputStream(
            samplerate=SAMPLE_RATE, channels=1, dtype="float32", callback=_audio_cb,
        )
        self.stream.start()
        self._call_after(lambda: self._ui_show_recording(mode))
        print(f"[wf] Recording ({mode}) ...")

    def _stop_recording(self):
        self.is_recording = False
        self._recording_mode = None

        if self.stream:
            self.stream.stop()
            self.stream.close()
            self.stream = None

        self._call_after(self._stop_amp_timer)

        if not self.audio_buffer:
            self._call_after(self._ui_hide)
            return

        audio = np.concatenate(self.audio_buffer).flatten()
        self.audio_buffer = []
        duration = len(audio) / SAMPLE_RATE
        print(f"[wf] Captured {duration:.1f}s")

        if duration < 0.3:
            self._call_after(self._ui_hide)
            return

        self.is_processing = True
        self._call_after(self._ui_show_processing)
        threading.Thread(
            target=self._transcribe_and_paste, args=(audio, duration), daemon=True
        ).start()

    # ── Transcription ────────────────────────────────────────────────

    def _transcribe_and_paste(self, audio: np.ndarray, rec_duration: float):
        try:
            t0 = time.time()
            text = transcribe(self.whisper_model, audio, self.device)
            if text is None:
                print("[wf] No speech detected")
                return
            print(f"[wf] {time.time() - t0:.1f}s  ->  {text}")
            append_history(text, rec_duration)
            copy_to_clipboard(text)
            time.sleep(0.05)
            simulate_paste()
        except Exception as exc:
            print(f"[wf] Error: {exc}")
        finally:
            self.is_processing = False
            self._call_after(self._ui_hide)

    # ── UI ───────────────────────────────────────────────────────────

    def _cancel_recording(self):
        if self.is_recording:
            self._stop_recording()

    def _ui_show_recording(self, mode: str):
        if self.overlay:
            if mode == "hold":
                self.overlay.show_hold()
            else:
                self.overlay.show_toggle()
        if self._icon:
            self._icon.icon = _ICON_RECORDING
            self._icon.title = "WisperFlow (recording)"
        self._start_amp_timer()

    def _ui_show_processing(self):
        if self.overlay:
            self.overlay.show_processing()
        if self._icon:
            self._icon.icon = _ICON_PROCESSING
            self._icon.title = "WisperFlow (processing)"

    def _ui_hide(self):
        if self.overlay:
            self.overlay.hide()
        if self._icon:
            self._icon.icon = _ICON_IDLE
            self._icon.title = "WisperFlow"

    def _start_amp_timer(self):
        self._stop_amp_timer()
        self._tick_amp()

    def _tick_amp(self):
        if self.is_recording and self.overlay:
            self.overlay.update_amplitude(self._latest_rms)
            self._amp_after_id = self._tk_root.after(70, self._tick_amp)

    def _stop_amp_timer(self):
        if self._amp_after_id is not None:
            self._tk_root.after_cancel(self._amp_after_id)
            self._amp_after_id = None
