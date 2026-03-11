"""Entry point: python -m wisperflow"""

import os
import sys


def main():
    os.environ["PYTHONUNBUFFERED"] = "1"
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(line_buffering=True)

    if getattr(sys, "frozen", False):
        bundle_base = getattr(sys, "_MEIPASS", None)
        if bundle_base is None:
            exe_dir = os.path.dirname(os.path.abspath(sys.executable))
            parent = os.path.dirname(exe_dir)
            bundle_base = os.path.join(parent, "Resources")
        if bundle_base not in sys.path:
            sys.path.insert(0, bundle_base)
        from wisperflow.ipc import send_to_running_instance
        from wisperflow.app import WFApp

        if "--settings" in sys.argv:
            from wisperflow.ui import run_settings
            run_settings()
            sys.exit(0)
    else:
        from .ipc import send_to_running_instance
        from .app import WFApp

    if sys.platform == "darwin":
        try:
            from ApplicationServices import AXIsProcessTrustedWithOptions
            from CoreFoundation import kCFBooleanTrue
            trusted = AXIsProcessTrustedWithOptions({"AXTrustedCheckOptionPrompt": kCFBooleanTrue})
            if not trusted:
                print("[wf] Accessibility permission required — check System Settings > Privacy > Accessibility")
        except Exception:
            pass

    if send_to_running_instance("show_settings"):
        print("[wf] Another instance is running. Opening settings there.")
        sys.exit(0)

    print("[wf] Starting WisperFlow Alternative ...")
    WFApp().run()


if __name__ == "__main__":
    main()
