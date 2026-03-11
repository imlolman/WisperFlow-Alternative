"""Shortcut parsing and CGEvent-based input listeners with event suppression."""

import threading

from pynput import keyboard
from Quartz import (
    CGEventGetIntegerValueField,
    CGEventMaskBit,
    CGEventTapCreate,
    CGEventTapEnable,
    CFMachPortCreateRunLoopSource,
    CFRunLoopAddSource,
    CFRunLoopGetCurrent,
    CFRunLoopRun,
    CFRunLoopStop,
    kCFRunLoopCommonModes,
    kCGEventFlagsChanged,
    kCGEventKeyDown,
    kCGEventKeyUp,
    kCGEventLeftMouseDown,
    kCGEventLeftMouseUp,
    kCGEventOtherMouseDown,
    kCGEventOtherMouseUp,
    kCGEventRightMouseDown,
    kCGEventRightMouseUp,
    kCGHeadInsertEventTap,
    kCGKeyboardEventKeycode,
    kCGMouseEventButtonNumber,
    kCGSessionEventTap,
)

# Maps config key names → pynput Key objects (used for vk extraction only)
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

CG_BTN_NAMES = {0: "left", 1: "right", 2: "middle", 3: "back", 4: "forward"}


def parse_shortcut(shortcut_str: str):
    """Parse 'mouse:back' or 'key:Alt_R' into (kind, target).

    For mouse shortcuts, target is the button name string.
    For key shortcuts, target is a pynput Key/KeyCode object.
    """
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


def vk_from_pynput(key) -> int | None:
    """Extract macOS virtual key code from a pynput key object."""
    if isinstance(key, keyboard.Key):
        return key.value.vk
    if isinstance(key, keyboard.KeyCode):
        return key.vk
    return None


class RawMouseListener:
    """CGEvent-based listener that distinguishes all mouse buttons and suppresses matched ones."""

    def __init__(self, callback, suppress_buttons=frozenset()):
        self._cb = callback
        self._suppress = suppress_buttons
        self._runloop = None

    def start(self):
        threading.Thread(target=self._run, daemon=True).start()

    def stop(self):
        rl = self._runloop
        if rl is not None:
            CFRunLoopStop(rl)
            self._runloop = None

    def _run(self):
        press_types = frozenset((kCGEventLeftMouseDown, kCGEventRightMouseDown, kCGEventOtherMouseDown))
        suppress = self._suppress
        mask = 0
        for e in (kCGEventLeftMouseDown, kCGEventLeftMouseUp,
                  kCGEventRightMouseDown, kCGEventRightMouseUp,
                  kCGEventOtherMouseDown, kCGEventOtherMouseUp):
            mask |= CGEventMaskBit(e)

        def cg_callback(proxy, etype, event, refcon):
            try:
                if etype in (kCGEventLeftMouseDown, kCGEventLeftMouseUp):
                    name = "left"
                elif etype in (kCGEventRightMouseDown, kCGEventRightMouseUp):
                    name = "right"
                else:
                    btn_num = CGEventGetIntegerValueField(event, kCGMouseEventButtonNumber)
                    name = CG_BTN_NAMES.get(btn_num, str(btn_num))
                self._cb(name, etype in press_types)
                if name in suppress:
                    return None
            except Exception:
                pass
            return event

        tap = CGEventTapCreate(
            kCGSessionEventTap, kCGHeadInsertEventTap,
            0x00000000,  # kCGEventTapOptionDefault — can suppress events
            mask, cg_callback, None,
        )
        if tap is None:
            print("[wf] CGEventTap (mouse) failed — grant Accessibility permission")
            return

        src = CFMachPortCreateRunLoopSource(None, tap, 0)
        self._runloop = CFRunLoopGetCurrent()
        CFRunLoopAddSource(self._runloop, src, kCFRunLoopCommonModes)
        CGEventTapEnable(tap, True)
        CFRunLoopRun()


class RawKeyboardListener:
    """CGEvent-based keyboard listener that suppresses matched shortcut key events."""

    def __init__(self, on_press, on_release, suppress_vks=frozenset()):
        self._on_press = on_press
        self._on_release = on_release
        self._suppress = suppress_vks
        self._runloop = None
        self._mod_pressed = set()

    def start(self):
        threading.Thread(target=self._run, daemon=True).start()

    def stop(self):
        rl = self._runloop
        if rl is not None:
            CFRunLoopStop(rl)
            self._runloop = None

    def _run(self):
        suppress = self._suppress
        mod_pressed = self._mod_pressed
        mask = (CGEventMaskBit(kCGEventKeyDown) |
                CGEventMaskBit(kCGEventKeyUp) |
                CGEventMaskBit(kCGEventFlagsChanged))

        def cg_callback(proxy, etype, event, refcon):
            try:
                vk = CGEventGetIntegerValueField(event, kCGKeyboardEventKeycode)
                if etype == kCGEventKeyDown:
                    self._on_press(vk)
                    if vk in suppress:
                        return None
                elif etype == kCGEventKeyUp:
                    self._on_release(vk)
                    if vk in suppress:
                        return None
                elif etype == kCGEventFlagsChanged:
                    if vk in mod_pressed:
                        mod_pressed.discard(vk)
                        self._on_release(vk)
                    else:
                        mod_pressed.add(vk)
                        self._on_press(vk)
                    if vk in suppress:
                        return None
            except Exception:
                pass
            return event

        tap = CGEventTapCreate(
            kCGSessionEventTap, kCGHeadInsertEventTap,
            0x00000000,  # kCGEventTapOptionDefault — can suppress events
            mask, cg_callback, None,
        )
        if tap is None:
            print("[wf] CGEventTap (keyboard) failed — grant Accessibility permission")
            return

        src = CFMachPortCreateRunLoopSource(None, tap, 0)
        self._runloop = CFRunLoopGetCurrent()
        CFRunLoopAddSource(self._runloop, src, kCFRunLoopCommonModes)
        CGEventTapEnable(tap, True)
        CFRunLoopRun()
