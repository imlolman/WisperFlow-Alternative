"""Single-instance TCP guard and inter-process command server."""

import socket

from .config import TCP_PORT


def send_to_running_instance(cmd: str = "show_settings") -> bool:
    try:
        s = socket.create_connection(("127.0.0.1", TCP_PORT), timeout=1)
        s.sendall(cmd.encode())
        s.close()
        return True
    except (ConnectionRefusedError, OSError):
        return False


def start_command_server(call_after, app):
    """Blocking loop — run in a daemon thread."""
    srv = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    srv.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    srv.bind(("127.0.0.1", TCP_PORT))
    srv.listen(2)
    srv.settimeout(1.0)
    while True:
        try:
            conn, _ = srv.accept()
            data = conn.recv(1024).decode().strip()
            conn.close()
            if data == "show_settings":
                call_after(app.open_settings)
                call_after(app._show_tray_temporarily)
        except socket.timeout:
            continue
        except Exception:
            continue
