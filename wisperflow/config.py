"""Configuration and history persistence."""

import json
from datetime import datetime
from pathlib import Path

CONFIG_PATH = Path.home() / ".wisper_config.json"
HISTORY_PATH = Path.home() / ".wisper_history.json"
SAMPLE_RATE = 16000
TCP_PORT = 19876

DEFAULT_CONFIG = {
    "shortcut_hold": "mouse:middle",
    "shortcut_toggle": "key:Alt_R",
    "hide_tray": False,
    "start_on_login": False,
    "mic_device": None,
    "setup_complete": False,
}


def load_config() -> dict:
    try:
        if CONFIG_PATH.exists():
            cfg = json.loads(CONFIG_PATH.read_text())
            migrated = False

            if "shortcut" in cfg and "shortcut_hold" not in cfg:
                mode = cfg.get("shortcut_mode", "hold")
                if mode == "toggle":
                    cfg["shortcut_toggle"] = cfg["shortcut"]
                    cfg["shortcut_hold"] = DEFAULT_CONFIG["shortcut_hold"]
                else:
                    cfg["shortcut_hold"] = cfg["shortcut"]
                    cfg["shortcut_toggle"] = DEFAULT_CONFIG["shortcut_toggle"]
                migrated = True

            for old in ("shortcut", "shortcut_mode", "shortcut_key", "hotkey", "model"):
                if old in cfg:
                    cfg.pop(old)
                    migrated = True

            if cfg.get("shortcut_hold") == cfg.get("shortcut_toggle"):
                cfg["shortcut_toggle"] = DEFAULT_CONFIG["shortcut_toggle"]
                migrated = True

            for k, v in DEFAULT_CONFIG.items():
                cfg.setdefault(k, v)

            if migrated:
                save_config(cfg)

            return cfg
    except Exception:
        pass
    return DEFAULT_CONFIG.copy()


def save_config(cfg: dict):
    CONFIG_PATH.write_text(json.dumps(cfg, indent=2))


def append_history(text: str, duration_s: float):
    history = []
    try:
        if HISTORY_PATH.exists():
            history = json.loads(HISTORY_PATH.read_text())
    except Exception:
        pass
    history.append({
        "timestamp": datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        "text": text,
        "duration_s": round(duration_s, 1),
    })
    HISTORY_PATH.write_text(json.dumps(history, indent=2))


def load_history() -> list:
    try:
        if HISTORY_PATH.exists():
            return json.loads(HISTORY_PATH.read_text())
    except Exception:
        pass
    return []
