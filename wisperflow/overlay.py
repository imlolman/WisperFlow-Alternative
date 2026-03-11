"""Waveform overlay window (black pill at screen bottom)."""

import threading
from pathlib import Path

import WebKit
from AppKit import NSBackingStoreBuffered, NSColor, NSPanel, NSScreen
from Foundation import NSObject as _NSObject, NSURL
from PyObjCTools import AppHelper

OVERLAY_HTML_PATH = Path(__file__).resolve().parent / "assets" / "overlay.html"


class _JSBridge(_NSObject):
    """Receives postMessage('cancel') from overlay JS."""
    _on_cancel = None

    def userContentController_didReceiveScriptMessage_(self, controller, message):
        if message.body() == "cancel" and self._on_cancel:
            AppHelper.callAfter(self._on_cancel)


class OverlayWindow:
    WIDTH = 90
    HEIGHT = 26

    def __init__(self, on_cancel=None):
        visible = NSScreen.mainScreen().visibleFrame()
        x = visible.origin.x + (visible.size.width - self.WIDTH) / 2
        y = visible.origin.y + 8

        self.window = NSPanel.alloc().initWithContentRect_styleMask_backing_defer_(
            ((x, y), (self.WIDTH, self.HEIGHT)),
            128,  # NSWindowStyleMaskNonactivatingPanel
            NSBackingStoreBuffered,
            False,
        )
        self.window.setLevel_(25)
        self.window.setFloatingPanel_(True)
        self.window.setBecomesKeyOnlyIfNeeded_(True)
        self.window.setOpaque_(False)
        self.window.setBackgroundColor_(NSColor.clearColor())
        self.window.setIgnoresMouseEvents_(True)
        self.window.setHasShadow_(True)
        self.window.setAlphaValue_(0.92)

        view = self.window.contentView()
        view.setWantsLayer_(True)
        view.layer().setCornerRadius_(self.HEIGHT / 2)
        view.layer().setMasksToBounds_(True)

        self._bridge = _JSBridge.alloc().init()
        self._bridge._on_cancel = on_cancel

        uc = WebKit.WKUserContentController.alloc().init()
        uc.addScriptMessageHandler_name_(self._bridge, "wf")

        conf = WebKit.WKWebViewConfiguration.alloc().init()
        conf.setUserContentController_(uc)

        self.webview = WebKit.WKWebView.alloc().initWithFrame_configuration_(
            ((0, 0), (self.WIDTH, self.HEIGHT)), conf
        )
        self.webview.setValue_forKey_(False, "drawsBackground")
        view.addSubview_(self.webview)

        html = OVERLAY_HTML_PATH.read_text()
        base_url = NSURL.fileURLWithPath_(str(OVERLAY_HTML_PATH.parent))
        self.webview.loadHTMLString_baseURL_(html, base_url)

        self._ready = False
        t = threading.Timer(0.5, self._mark_ready)
        t.daemon = True
        t.start()

    def _mark_ready(self):
        self._ready = True

    def _js(self, code: str):
        if self._ready:
            self.webview.evaluateJavaScript_completionHandler_(code, None)

    def show_hold(self):
        self.window.setIgnoresMouseEvents_(False)
        self.window.orderFront_(None)
        self._js("showHold()")

    def show_toggle(self):
        self.window.setIgnoresMouseEvents_(False)
        self.window.orderFront_(None)
        self._js("showToggle()")

    def show_processing(self):
        self.window.setIgnoresMouseEvents_(True)
        self._js("showProcessing()")

    def update_amplitude(self, rms: float):
        a = min(1.0, rms * 20)
        self._js(f"setAmplitude({a:.3f})")

    def hide(self):
        self.window.setIgnoresMouseEvents_(True)
        self.window.orderOut_(None)
