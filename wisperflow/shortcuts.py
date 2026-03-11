"""Cross-platform shortcut listening via pynput."""

from pynput import keyboard, mouse

_KEYSYM_TO_PYNPUT_KEY = {
    "Alt_R": keyboard.Key.alt_r,
    "Alt_L": keyboard.Key.alt,
    "Control_R": keyboard.Key.ctrl_r,
    "Control_L": keyboard.Key.ctrl,
    "Super_R": keyboard.Key.cmd_r,
    "Super_L": keyboard.Key.cmd,
    "Meta_R": keyboard.Key.cmd_r,
    "Meta_L": keyboard.Key.cmd,
    "Shift_R": keyboard.Key.shift_r,
    "Shift_L": keyboard.Key.shift,
    "Caps_Lock": keyboard.Key.caps_lock,
    "Escape": keyboard.Key.esc,
    "space": keyboard.Key.space,
    "Tab": keyboard.Key.tab,
    "BackSpace": keyboard.Key.backspace,
    "Return": keyboard.Key.enter,
    "Delete": keyboard.Key.delete,
    "Home": keyboard.Key.home,
    "End": keyboard.Key.end,
    **{f"F{i}": getattr(keyboard.Key, f"f{i}") for i in range(1, 21)},
}

_PYNPUT_BTN_NAMES = {
    mouse.Button.left: "left",
    mouse.Button.right: "right",
    mouse.Button.middle: "middle",
    mouse.Button.x1: "back",
    mouse.Button.x2: "forward",
}


def parse_shortcut(shortcut_str: str):
    """Parse 'mouse:back' or 'key:Alt_R' into (kind, target)."""
    if not shortcut_str:
        return (None, None)
    if ":" not in shortcut_str:
        shortcut_str = f"key:{shortcut_str}"
    kind, name = shortcut_str.split(":", 1)
    if kind == "mouse":
        return ("mouse", name)
    if name in _KEYSYM_TO_PYNPUT_KEY:
        return ("key", _KEYSYM_TO_PYNPUT_KEY[name])
    if len(name) == 1:
        return ("key", keyboard.KeyCode.from_char(name.lower()))
    return ("key", None)


def keys_match(key, target) -> bool:
    if key == target:
        return True
    if isinstance(key, keyboard.KeyCode) and isinstance(target, keyboard.KeyCode):
        return bool(key.char and target.char and key.char == target.char)
    return False


class RawKeyboardListener:
    def __init__(self, on_press, on_release, suppress_vks=frozenset(), on_tap_failed=None):
        self._on_press = on_press
        self._on_release = on_release
        self._listener = None

    def start(self):
        def _press(key):
            try:
                self._on_press(key)
            except Exception:
                pass

        def _release(key):
            try:
                self._on_release(key)
            except Exception:
                pass

        self._listener = keyboard.Listener(on_press=_press, on_release=_release)
        self._listener.daemon = True
        self._listener.start()

    def stop(self):
        if self._listener:
            try:
                self._listener.stop()
            except Exception:
                pass
            self._listener = None


class RawMouseListener:
    def __init__(self, callback, suppress_buttons=frozenset(), on_tap_failed=None):
        self._cb = callback
        self._listener = None

    def start(self):
        def _click(x, y, button, pressed):
            try:
                name = _PYNPUT_BTN_NAMES.get(button, str(button))
                self._cb(name, pressed)
            except Exception:
                pass

        self._listener = mouse.Listener(on_click=_click)
        self._listener.daemon = True
        self._listener.start()

    def stop(self):
        if self._listener:
            try:
                self._listener.stop()
            except Exception:
                pass
            self._listener = None
