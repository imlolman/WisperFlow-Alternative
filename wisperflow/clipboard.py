"""Type text directly via CGEvent unicode injection (no clipboard)."""

import time

from Quartz import (
    CGEventCreateKeyboardEvent,
    CGEventKeyboardSetUnicodeString,
    CGEventPost,
    kCGHIDEventTap,
)

_INTER_CHAR_DELAY = 0.0


def type_text(text: str):
    """Inject text as synthetic key events without touching the clipboard."""
    time.sleep(0.05)
    for char in text:
        down = CGEventCreateKeyboardEvent(None, 0, True)
        CGEventKeyboardSetUnicodeString(down, 1, char)
        CGEventPost(kCGHIDEventTap, down)
        up = CGEventCreateKeyboardEvent(None, 0, False)
        CGEventKeyboardSetUnicodeString(up, 1, char)
        CGEventPost(kCGHIDEventTap, up)
        if _INTER_CHAR_DELAY:
            time.sleep(_INTER_CHAR_DELAY)
