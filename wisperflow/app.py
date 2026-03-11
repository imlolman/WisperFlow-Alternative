"""WisperFlow Alternative — main tray application."""

import subprocess
import sys
import threading
import time

import numpy as np
import rumps
import sounddevice as sd
from PyObjCTools import AppHelper

from .clipboard import copy_to_clipboard, simulate_paste
from .config import CONFIG_PATH, SAMPLE_RATE, load_config, append_history
from .ipc import start_command_server
from .overlay import OverlayWindow
from .shortcuts import (
    RawKeyboardListener,
    RawMouseListener,
    parse_shortcut,
    vk_from_pynput,
)
from .transcriber import get_device, load_model, transcribe

_PROJECT_ROOT = __import__("pathlib").Path(__file__).resolve().parent.parent


class WFApp(rumps.App):
    def __init__(self):
        self.config = load_config()
        super().__init__("WisperFlow Alternative", title="⏳", quit_button=None)

        self.menu = [
            rumps.MenuItem("Settings", callback=lambda _: self.open_settings()),
            None,
            rumps.MenuItem("Quit", callback=lambda _: rumps.quit_application()),
        ]

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
        self._hold_release_timer: threading.Timer | None = None
        self._config_mtime: float = 0
        self._settings_proc: subprocess.Popen | None = None
        self._tray_hide_timer: threading.Timer | None = None
        self._latest_rms: float = 0.0
        self._amplitude_timer: rumps.Timer | None = None

        threading.Thread(target=start_command_server, args=(self,), daemon=True).start()
        threading.Thread(target=self._load_model, daemon=True).start()
        threading.Thread(target=self._watch_config, daemon=True).start()

        if self.config.get("hide_tray"):
            self._schedule_tray_hide()

    # ── Tray auto-hide ───────────────────────────────────────────────

    def _schedule_tray_hide(self):
        if self._tray_hide_timer is not None:
            self._tray_hide_timer.cancel()
        self._tray_hide_timer = threading.Timer(10.0, self._hide_tray)
        self._tray_hide_timer.daemon = True
        self._tray_hide_timer.start()

    def _hide_tray(self):
        if self.config.get("hide_tray"):
            AppHelper.callAfter(self._do_hide_tray)

    def _do_hide_tray(self):
        try:
            self._nsapp.nsstatusitem.setVisible_(False)
        except Exception:
            pass

    def _show_tray_temporarily(self):
        try:
            self._nsapp.nsstatusitem.setVisible_(True)
        except Exception:
            pass
        if self.config.get("hide_tray"):
            self._schedule_tray_hide()

    def _force_show_tray(self):
        if self._tray_hide_timer is not None:
            self._tray_hide_timer.cancel()
            self._tray_hide_timer = None
        try:
            self._nsapp.nsstatusitem.setVisible_(True)
        except Exception:
            pass

    # ── Settings ─────────────────────────────────────────────────────

    def open_settings(self):
        self._show_tray_temporarily()
        if self._settings_proc and self._settings_proc.poll() is None:
            try:
                subprocess.Popen([
                    "osascript", "-e",
                    'tell application "System Events" to set frontmost of '
                    f'(first process whose unix id is {self._settings_proc.pid}) to true'
                ])
            except Exception:
                pass
            return
        self._settings_proc = subprocess.Popen(
            [sys.executable, "-m", "wisperflow.ui"], cwd=str(_PROJECT_ROOT),
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

                        old_hide = old.get("hide_tray", False)
                        new_hide = new.get("hide_tray", False)
                        if old_hide != new_hide:
                            if new_hide:
                                self._schedule_tray_hide()
                                print("[wf] Tray auto-hide enabled")
                            else:
                                AppHelper.callAfter(self._force_show_tray)
                                print("[wf] Tray icon shown")
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
        AppHelper.callAfter(lambda: setattr(self, "title", "𝗪"))
        self._restart_listeners()

    # ── Listener management ──────────────────────────────────────────

    def _stop_all_listeners(self):
        for attr in ("_key_listener", "_mouse_listener"):
            listener = getattr(self, attr, None)
            if listener is not None:
                listener.stop()
                setattr(self, attr, None)

    def _restart_listeners(self):
        self._stop_all_listeners()
        self._hold_release_timer = None

        hold_kind, hold_target = parse_shortcut(self.config["shortcut_hold"])
        toggle_kind, toggle_target = parse_shortcut(self.config["shortcut_toggle"])

        has_key_hold = hold_kind == "key" and hold_target is not None
        has_key_toggle = toggle_kind == "key" and toggle_target is not None
        has_mouse_hold = hold_kind == "mouse" and hold_target is not None
        has_mouse_toggle = toggle_kind == "mouse" and toggle_target is not None

        # Shared hold logic: double-press converts hold → toggle
        def hold_press():
            if self.whisper_model is None:
                return
            timer = self._hold_release_timer
            if timer is not None:
                timer.cancel()
                self._hold_release_timer = None
                self._recording_mode = "toggle"
                AppHelper.callAfter(lambda: self.overlay and self.overlay.show_toggle())
                return
            if self.is_recording and self._recording_mode == "toggle":
                self._stop_recording()
            elif not self.is_recording and not self.is_processing:
                self._start_recording("hold")

        def hold_release():
            if not (self.is_recording and self._recording_mode == "hold"):
                return

            def _finalize():
                self._hold_release_timer = None
                if self.is_recording and self._recording_mode == "hold":
                    self._stop_recording()

            t = threading.Timer(0.3, _finalize)
            t.daemon = True
            self._hold_release_timer = t
            t.start()

        def toggle_press():
            if self.whisper_model is None:
                return
            if self.is_recording and self._recording_mode == "toggle":
                self._stop_recording()
            elif not self.is_recording and not self.is_processing:
                self._start_recording("toggle")

        # Keyboard listener
        if has_key_hold or has_key_toggle:
            vk_hold = vk_from_pynput(hold_target) if has_key_hold else None
            vk_toggle = vk_from_pynput(toggle_target) if has_key_toggle else None
            suppress_vks = {vk for vk in (vk_hold, vk_toggle) if vk is not None}

            def on_key_press(vk):
                if vk_hold is not None and vk == vk_hold:
                    hold_press()
                elif vk_toggle is not None and vk == vk_toggle:
                    toggle_press()

            def on_key_release(vk):
                if vk_hold is not None and vk == vk_hold:
                    hold_release()

            self._key_listener = RawKeyboardListener(on_key_press, on_key_release, suppress_vks)
            self._key_listener.start()

        # Mouse listener
        if has_mouse_hold or has_mouse_toggle:
            mt_hold = hold_target if has_mouse_hold else None
            mt_toggle = toggle_target if has_mouse_toggle else None
            suppress_btns = {b for b in (mt_hold, mt_toggle) if b is not None}

            def on_mouse(btn_name, pressed):
                hold_match = mt_hold is not None and btn_name == mt_hold
                toggle_match = mt_toggle is not None and btn_name == mt_toggle
                if hold_match:
                    if pressed:
                        hold_press()
                    else:
                        hold_release()
                elif toggle_match and pressed:
                    toggle_press()

            self._mouse_listener = RawMouseListener(on_mouse, suppress_btns)
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
            samplerate=SAMPLE_RATE, channels=1, dtype="float32", callback=_audio_cb
        )
        self.stream.start()
        AppHelper.callAfter(lambda: self._ui_show_recording(mode))
        print(f"[wf] Recording ({mode}) ...")

    def _stop_recording(self):
        self.is_recording = False
        self._recording_mode = None

        if self.stream is not None:
            self.stream.stop()
            self.stream.close()
            self.stream = None

        AppHelper.callAfter(self._stop_amplitude_timer)

        if not self.audio_buffer:
            AppHelper.callAfter(self._ui_hide)
            return

        audio = np.concatenate(self.audio_buffer).flatten()
        self.audio_buffer = []
        duration = len(audio) / SAMPLE_RATE
        print(f"[wf] Captured {duration:.1f}s")

        if duration < 0.3:
            AppHelper.callAfter(self._ui_hide)
            return

        self.is_processing = True
        AppHelper.callAfter(self._ui_show_processing)
        threading.Thread(
            target=self._transcribe_and_paste, args=(audio, duration), daemon=True
        ).start()

    # ── Transcription + paste ────────────────────────────────────────

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
            AppHelper.callAfter(self._ui_hide)

    # ── UI / overlay ─────────────────────────────────────────────────

    def _cancel_recording(self):
        if self.is_recording:
            if self._hold_release_timer:
                self._hold_release_timer.cancel()
                self._hold_release_timer = None
            self._stop_recording()

    def _ui_show_recording(self, mode: str):
        if self.overlay is None:
            self.overlay = OverlayWindow(on_cancel=self._cancel_recording)
        if mode == "hold":
            self.overlay.show_hold()
        else:
            self.overlay.show_toggle()
        self.title = "◉"
        self._start_amplitude_timer()

    def _ui_show_processing(self):
        if self.overlay is not None:
            self.overlay.show_processing()
        self.title = "⏳"

    def _ui_hide(self):
        if self.overlay is not None:
            self.overlay.hide()
        self.title = "𝗪"

    def _start_amplitude_timer(self):
        if self._amplitude_timer is not None:
            self._amplitude_timer.stop()
        self._amplitude_timer = rumps.Timer(self._tick_amplitude, 0.07)
        self._amplitude_timer.start()

    def _stop_amplitude_timer(self):
        if self._amplitude_timer is not None:
            self._amplitude_timer.stop()
            self._amplitude_timer = None

    def _tick_amplitude(self, _timer):
        if self.overlay and self.is_recording:
            self.overlay.update_amplitude(self._latest_rms)
