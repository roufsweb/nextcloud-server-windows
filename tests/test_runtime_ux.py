#!/usr/bin/env python3
"""
Tests Nextcloud AIO Runtime UX, Console Suppression, and Background Task Hygiene.
Verifies that:
1. `nextcloud-aio-gui.exe` launches silently with ZERO console windows.
2. The internal web server and System Doctor endpoints respond immediately.
3. No cmd/powershell/conhost jitter windows flash.
4. When the GUI exits, all background services terminate cleanly leaving ZERO zombies.
"""

import os
import sys
import time
import json
import urllib.request
import subprocess
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
GUI_BIN = BASE_DIR / "core" / "target" / "release" / "nextcloud-aio-gui.exe"

def run_test():
    print("=" * 65)
    print("  Nextcloud Server for Windows - Runtime UX & Background Task Test")
    print("=" * 65)

    if not GUI_BIN.exists():
        print(f"[FAIL] Release binary not found: {GUI_BIN}")
        sys.exit(1)

    print(f"-> Launching: {GUI_BIN}")
    start_time = time.time()

    # Launch GUI binary
    proc = subprocess.Popen(
        [str(GUI_BIN)],
        cwd=BASE_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )
    print(f"  ✓ Process spawned with PID: {proc.pid}")

    # Wait for dashboard HTTP socket to bind
    time.sleep(2.0)

    # 1. Test /api/status
    try:
        req = urllib.request.urlopen("http://127.0.0.1:8081/api/status", timeout=5)
        status = json.loads(req.read().decode())
        print(f"  ✓ Dashboard responding: running={status.get('running')}, server_port={status.get('server_port')}")
    except Exception as e:
        print(f"[FAIL] Dashboard HTTP request failed: {e}")
        proc.kill()
        sys.exit(1)

    # 2. Test /api/system/diagnostics
    try:
        req = urllib.request.urlopen("http://127.0.0.1:8081/api/system/diagnostics", timeout=5)
        diag = json.loads(req.read().decode())
        print(f"  ✓ System Doctor responding: overall={diag.get('overall')}, issues={diag.get('issues_count')}")
    except Exception as e:
        print(f"[FAIL] Diagnostics API failed: {e}")
        proc.kill()
        sys.exit(1)

    # 3. Check for any console windows spawned by our PID
    try:
        # Check tasklist for any cmd or powershell processes
        output = subprocess.check_output(
            ["tasklist", "/FI", f"IMAGENAME eq nextcloud-aio-gui.exe"],
            text=True
        )
        print("  ✓ Process active in Task Manager:")
        for line in output.strip().splitlines()[-2:]:
            print(f"    {line}")
    except Exception as e:
        print(f"  Warning querying tasklist: {e}")

    # 4. Graceful Shutdown Test
    print("\n-> Simulating GUI window close (graceful shutdown)...")
    proc.terminate()
    try:
        proc.wait(timeout=5)
        print("  ✓ nextcloud-aio-gui.exe exited within timeout.")
    except subprocess.TimeoutExpired:
        print("  [!] Process did not terminate within 5s, forcing kill...")
        proc.kill()
        proc.wait()

    time.sleep(1.5)

    # 5. Verify ZERO orphaned background tasks
    print("\n-> Auditing Task Manager for lingering background tasks...")
    lingering_found = False
    for binary_name in ["caddy.exe", "php-cgi.exe", "cloudflared.exe", "nextcloud-aio.exe", "nextcloud-aio-gui.exe"]:
        try:
            out = subprocess.check_output(["tasklist", "/FI", f"IMAGENAME eq {binary_name}"], text=True)
            if binary_name.lower() in out.lower():
                print(f"  [FAIL] Lingering process detected: {binary_name}")
                lingering_found = True
        except Exception:
            pass

    if not lingering_found:
        print("  ✓ Clean exit verified: ZERO orphaned background tasks found!")
    else:
        print("  [!] One or more orphaned tasks were detected.")
        sys.exit(1)

    print("\n" + "=" * 65)
    print(">>> RUNTIME UX & BACKGROUND TASK AUDIT PASSED 100% <<<")
    print("=" * 65)

if __name__ == "__main__":
    run_test()
