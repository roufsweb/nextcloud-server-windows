#!/usr/bin/env python3
"""
Captures high-resolution screenshots of the Nextcloud AIO Desktop UI views.
Uses a temporary HTTP server and headless Chrome with virtual-time-budget
to ensure full JavaScript execution and SVG gauge rendering.
"""

import os
import subprocess
import time
import signal
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
UI_DIR = BASE_DIR / "ui"
OUTPUT_DIR = BASE_DIR / "docs" / "img" / "screenshots"

VIEWS = [
    ("01_dashboard.png", "view-dashboard", "Dashboard Overview & Service Cards"),
    ("02_storage_drives.png", "view-storage", "Storage & Drives (Direct NTFS Access)"),
    ("03_cloudflare_tunnel.png", "view-tunnel", "Cloudflare Tunnel Token Configuration"),
    ("04_users_security.png", "view-users", "Users & Administrative Security"),
    ("05_in_app_logs.png", "view-logs", "In-App Real-Time Log Console"),
]

def main():
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    port = 8089

    print("-> Starting temporary UI HTTP server on port", port)
    server = subprocess.Popen(
        ["python3", "-m", "http.server", str(port), "--directory", str(UI_DIR)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )

    try:
        time.sleep(1.2)  # Wait for server to bind

        for filename, view_id, label in VIEWS:
            out_path = OUTPUT_DIR / filename
            url = f"http://127.0.0.1:{port}/#{view_id}"
            print(f"-> Capturing {label} -> {out_path.name}...")

            cmd = [
                "google-chrome-stable",
                "--headless",
                "--disable-gpu",
                "--no-sandbox",
                "--window-size=1240,840",
                "--virtual-time-budget=2500",
                f"--screenshot={out_path}",
                url
            ]

            res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            if res.returncode == 0 and out_path.exists():
                size_kb = out_path.stat().st_size // 1024
                print(f"   ✓ {filename} captured ({size_kb} KB)")
            else:
                print(f"   ✗ Failed to capture {filename}")

    finally:
        print("-> Stopping temporary HTTP server...")
        server.terminate()
        server.wait()

    print("\n✓ All screenshots captured successfully in:", OUTPUT_DIR)

if __name__ == "__main__":
    main()
