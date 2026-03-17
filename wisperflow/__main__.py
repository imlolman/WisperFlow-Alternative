"""Entry point: python -m wisperflow"""

import os
import sys

# PyInstaller sees this and adds wisperflow.ui to the bundle; never executed
if False:
    import wisperflow.ui  # noqa: F401


def main():
    os.environ["PYTHONUNBUFFERED"] = "1"
    if hasattr(sys.stdout, "reconfigure"):
        sys.stdout.reconfigure(line_buffering=True)

    frozen = getattr(sys, "frozen", False)

    if frozen:
        bundle_base = getattr(sys, "_MEIPASS", None)
        if bundle_base is None:
            exe_dir = os.path.dirname(os.path.abspath(sys.executable))
            parent = os.path.dirname(exe_dir)
            bundle_base = os.path.join(parent, "Resources")
        if bundle_base not in sys.path:
            sys.path.insert(0, bundle_base)
        from wisperflow.config import load_config
        from wisperflow.ipc import send_to_running_instance
        from wisperflow.app import WFApp

        if "--settings" in sys.argv:
            from wisperflow.ui import run_settings
            run_settings()
            sys.exit(0)

        if "--onboarding" in sys.argv:
            from wisperflow.onboarding import run_onboarding
            run_onboarding()
            sys.exit(0)
    else:
        from .config import load_config
        from .ipc import send_to_running_instance
        from .app import WFApp

    try:
        from ApplicationServices import AXIsProcessTrustedWithOptions
        from CoreFoundation import kCFBooleanTrue
        trusted = AXIsProcessTrustedWithOptions({"AXTrustedCheckOptionPrompt": kCFBooleanTrue})
        if not trusted:
            print("[wf] Accessibility permission required — check System Settings > Privacy > Accessibility")
    except Exception:
        pass

    cfg = load_config()
    if not cfg.get("setup_complete", False):
        print("[wf] First launch — running onboarding ...")
        if frozen:
            import subprocess
            subprocess.run([sys.executable, "--onboarding"])
        else:
            from .onboarding import run_onboarding
            run_onboarding()
        cfg = load_config()
        if not cfg.get("setup_complete", False):
            print("[wf] Onboarding not completed, exiting.")
            sys.exit(0)
        # Relaunch cleanly so macOS app lifecycle starts fresh
        print("[wf] Onboarding done — relaunching ...")
        os.execv(sys.executable, [sys.executable] + [a for a in sys.argv[1:] if a != "--onboarding"])

    if send_to_running_instance("show_settings"):
        print("[wf] Another instance is running. Opening settings there.")
        sys.exit(0)

    print("[wf] Starting WisperFlow Alternative ...")
    WFApp().run()


if __name__ == "__main__":
    main()
