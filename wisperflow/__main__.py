"""Entry point: python -m wisperflow"""

import os
import sys


def main():
    os.environ["PYTHONUNBUFFERED"] = "1"
    sys.stdout.reconfigure(line_buffering=True)

    try:
        import HIServices
        HIServices.AXIsProcessTrusted()
    except Exception:
        pass

    from .ipc import send_to_running_instance

    if send_to_running_instance("show_settings"):
        print("[wf] Another instance is running. Opening settings there.")
        sys.exit(0)

    print("[wf] Starting WisperFlow Alternative ...")
    from .app import WFApp
    WFApp().run()


if __name__ == "__main__":
    main()
