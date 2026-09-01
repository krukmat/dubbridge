from __future__ import annotations

import json
import socket
import urllib.request


def unload_model(host, model, timeout=10):
    if not model:
        return False
    if host and "://" not in host:
        host = f"http://{host}"
    url = f"{host.rstrip('/')}/api/generate"
    payload = json.dumps({"model": model, "keep_alive": 0}).encode("utf-8")
    request = urllib.request.Request(
        url,
        data=payload,
        method="POST",
        headers={"Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            response.read()
        return True
    except (OSError, TimeoutError, socket.timeout):
        return False
