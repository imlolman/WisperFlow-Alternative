"""Clipboard copy and Cmd+V paste simulation."""

import subprocess
import time

from Quartz import (
    CGEventCreateKeyboardEvent,
    CGEventPost,
    CGEventSetFlags,
    kCGEventFlagMaskCommand,
    kCGHIDEventTap,
)


def copy_to_clipboard(text: str):
    subprocess.run(["pbcopy"], input=text.encode("utf-8"), check=True)


def simulate_paste():
    time.sleep(0.05)
    v_keycode = 0x09
    down = CGEventCreateKeyboardEvent(None, v_keycode, True)
    CGEventSetFlags(down, kCGEventFlagMaskCommand)
    up = CGEventCreateKeyboardEvent(None, v_keycode, False)
    CGEventSetFlags(up, kCGEventFlagMaskCommand)
    CGEventPost(kCGHIDEventTap, down)
    CGEventPost(kCGHIDEventTap, up)
