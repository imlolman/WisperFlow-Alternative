"""Frameless floating overlay using tkinter (cross-platform)."""

import sys
import tkinter as tk


class OverlayWindow:
    WIDTH = 140
    HEIGHT = 28

    def __init__(self, root: tk.Tk, on_cancel=None):
        self._root = root
        self._on_cancel = on_cancel
        self._win: tk.Toplevel | None = None
        self._label: tk.Label | None = None
        self._after_id = None

    def _create(self):
        if self._win is not None:
            return
        self._win = tk.Toplevel(self._root)
        self._win.overrideredirect(True)
        self._win.attributes("-topmost", True)
        self._win.attributes("-alpha", 0.92)
        if sys.platform == "darwin":
            self._win.attributes("-transparent", True)
        self._win.configure(bg="#1c1c1c")

        sw = self._root.winfo_screenwidth()
        sh = self._root.winfo_screenheight()
        x = (sw - self.WIDTH) // 2
        y = sh - self.HEIGHT - 50
        self._win.geometry(f"{self.WIDTH}x{self.HEIGHT}+{x}+{y}")

        self._label = tk.Label(
            self._win, text="", fg="white", bg="#1c1c1c",
            font=("Helvetica", 10, "bold"),
        )
        self._label.pack(expand=True, fill="both", padx=6, pady=2)
        self._win.withdraw()

    def _show(self, text: str):
        self._create()
        self._label.config(text=text)
        self._win.deiconify()
        self._win.lift()

    def show_hold(self):
        self._show("● Hold to record")

    def show_toggle(self):
        self._show("● Recording")

    def show_processing(self):
        self._create()
        if self._label:
            self._label.config(text="⏳ Processing…")

    def update_amplitude(self, rms: float):
        if self._win and self._win.winfo_ismapped():
            bars = int(min(rms * 20, 1.0) * 6)
            self._label.config(text="● " + "▪" * bars + "▫" * (6 - bars))

    def hide(self):
        if self._win:
            self._win.withdraw()
