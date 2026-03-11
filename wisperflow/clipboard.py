"""Clipboard copy and paste simulation (cross-platform)."""

import sys
import time

import pyperclip
from pynput.keyboard import Controller, Key

_keyboard = Controller()


def copy_to_clipboard(text: str):
    pyperclip.copy(text)


def simulate_paste():
    time.sleep(0.05)
    paste_key = Key.cmd if sys.platform == "darwin" else Key.ctrl
    with _keyboard.pressed(paste_key):
        _keyboard.press("v")
        _keyboard.release("v")
