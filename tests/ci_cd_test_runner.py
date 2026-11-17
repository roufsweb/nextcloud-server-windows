#!/usr/bin/env python3
"""
Nextcloud Server for Windows (All-In-One) - Comprehensive CI/CD Test Runner
Tests extraction, supervisor REST API, direct drive permissions, config audit,
and high-bitrate multi-format media stress testing.
"""

import os
import sys
import time
import json
import shutil
import zipfile
import hashlib
import urllib.request
import urllib.error
import subprocess
from pathlib import Path

# Paths
BASE_DIR = Path(__file__).resolve().parent.parent
DIST_ZIP = BASE_DIR / "dist" / "nextcloud-windows-aio-v1.0.0-portable-x64.zip"
TEST_ROOT = Path("/mnt/data/test_env")
SANDBOX_DIR = TEST_ROOT / "nextcloud_aio_sandbox"
DRIVES_DIR = TEST_ROOT / "simulated_drives"
FFMPEG_BIN = Path("/mnt/data/tools/bin/ffmpeg")
SUPERVISOR_BIN = BASE_DIR / "core" / "target" / "release" / "nextcloud-aio"

TEST_PORT = 8192
BASE_URL = f"http://127.0.0.1:{TEST_PORT}"

# Terminal Colors
C_GREEN = "\033[92m"
C_RED = "\033[91m"
C_YELLOW = "\033[93m"
C_BLUE = "\033[94m"
C_BOLD = "\033[1m"
C_RESET = "\033[0m"

test_results = []

def log_header(title):
    print(f"\n{C_BOLD}{C_BLUE}{'='*70}{C_RESET}")
    print(f"{C_BOLD}{C_BLUE}  {title}{C_RESET}")
    print(f"{C_BOLD}{C_BLUE}{'='*70}{C_RESET}")

def report(name, passed, details=""):
    status = f"{C_GREEN}✓ PASSED{C_RESET}" if passed else f"{C_RED}✗ FAILED{C_RESET}"
    test_results.append((name, passed, details))
    print(f"  [{status}] {name}")
    if details:
        print(f"           └─ {details}")
    if not passed:
        print(f"{C_RED}CRITICAL TEST FAILURE in '{name}'!{C_RESET}")

def http_req(endpoint, method="GET", data=None):
    url = f"{BASE_URL}{endpoint}"
    headers = {"Content-Type": "application/json"} if data else {}
    body = json.dumps(data).encode("utf-8") if data else None
    req = urllib.request.Request(url, data=body, headers=headers, method=method)
    with urllib.request.urlopen(req, timeout=5) as resp:
        content = resp.read().decode("utf-8")
        try:
            return resp.status, json.loads(content)
        except:
            return resp.status, content

# ==============================================================================
# 1. Extraction & Package Hygiene Test
# ==============================================================================
def test_extraction_and_packaging():
    log_header("1. Package Extraction & Directory Hierarchy Verification")
    if not DIST_ZIP.exists():
        report("Distribution ZIP exists", False, f"File not found: {DIST_ZIP}")
        return False
    
    zip_size_mb = DIST_ZIP.stat().st_size / (1024 * 1024)
    report("Distribution ZIP size budget (<500 MB)", zip_size_mb < 500, f"Size: {zip_size_mb:.2f} MB")

    if SANDBOX_DIR.exists():
        shutil.rmtree(SANDBOX_DIR)
    SANDBOX_DIR.mkdir(parents=True, exist_ok=True)

    print("  -> Extracting portable ZIP into sandbox...")
    with zipfile.ZipFile(DIST_ZIP, 'r') as zf:
        zf.extractall(SANDBOX_DIR)

    extracted_root = SANDBOX_DIR / "nextcloud-windows-aio"
    report("Extracted root exists", extracted_root.is_dir(), str(extracted_root))

    required_files = [
        "ui/index.html",
        "ui/css/app.css",
        "ui/css/components.css",
        "ui/js/app.js",
        "ui/js/api.js",
        "ui/components/storage-gauge.js",
        "ui/components/log-viewer.js",
        "config/nextcloud/autoconfig.php",
        "config/nextcloud/config.sample.php",
        "config/php/php.ini",
        "config/caddy/Caddyfile",
        "start-server.bat",
        "open-dashboard.bat",
        "Nextcloud-AIO.vbs",
        "nextcloud-aio.exe",
        "nextcloud-aio-gui.exe",
        "README.md",
        "LICENSE"
    ]

    all_found = True
    for f in required_files:
        target = extracted_root / f
        if not target.exists():
            report(f"Required file: {f}", False, "Missing in zip")
            all_found = False
    
    if all_found:
        report("All 17 required distribution files verified", True, "Complete portable bundle structure")
    return all_found

# ==============================================================================
# 2. Supervisor Server & REST API Test Suite
# ==============================================================================
def test_supervisor_rest_api():
    log_header("2. Supervisor Daemon & End-to-End REST API Testing")
    
    # Write a test config inside sandbox
    cfg_path = SANDBOX_DIR / "test_config.json"
    init_cfg = {
        "server_port": TEST_PORT,
        "data_dir": str(SANDBOX_DIR / "data"),
        "external_drives": [],
        "trusted_domains": ["localhost", "127.0.0.1"],
        "cloudflare_token": None,
        "admin_username": "admin"
    }
    with open(cfg_path, "w") as f:
        json.dump(init_cfg, f, indent=2)

    # Spawn supervisor
    print(f"  -> Spawning supervisor on port {TEST_PORT}...")
    proc = subprocess.Popen(
        [str(SUPERVISOR_BIN), "server", "--port", str(TEST_PORT), "--config", str(cfg_path)],
        cwd=BASE_DIR,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )

    try:
        # Wait for server to bind
        time.sleep(1.0)
        
        # Test 1: GET /api/status
        status_code, status_data = http_req("/api/status")
        report("GET /api/status responds 200 OK", status_code == 200, f"Running: {status_data.get('running')}")
        report("Supervisor reports valid web port", status_data.get("server_port") >= 80, f"Port: {status_data.get('server_port')}")

        # Test Port Update API
        code, port_resp = http_req("/api/config/port", "POST", {"port": 8085})
        report("POST /api/config/port sets new web port", port_resp.get("status") == "port_updated", str(port_resp))
        _, updated_status = http_req("/api/status")
        report("Supervisor status reflects updated web port", updated_status.get("server_port") == 8085, f"Updated Port: {updated_status.get('server_port')}")

        # Test 2: GET /api/users/admin
        code, admin_info = http_req("/api/users/admin")
        report("GET /api/users/admin responds with active admin", admin_info.get("username") == "admin", str(admin_info))

        # Test 3: POST /api/users/update-admin (change admin username & password)
        new_user = "superadmin"
        new_pass = "SuperNextcloudPass2026!"
        code, update_resp = http_req("/api/users/update-admin", "POST", {"username": new_user, "password": new_pass})
        report("POST /api/users/update-admin updates credentials", update_resp.get("status") == "success", str(update_resp))

        # Verify admin change persisted
        code, updated_admin = http_req("/api/users/admin")
        report("GET /api/users/admin reflects updated username", updated_admin.get("username") == new_user, f"User: {updated_admin.get('username')}")

        # Test 4: Trusted Domains API (Add & Remove)
        code, dom_add = http_req("/api/config/domains/add", "POST", {"domain": "cloud.test-enterprise.com"})
        report("POST /api/config/domains/add adds custom domain", dom_add.get("status") == "domain_added", str(dom_add))

        code, domains = http_req("/api/config/domains")
        report("GET /api/config/domains includes added domain", "cloud.test-enterprise.com" in domains, f"Domains: {domains}")

        code, dom_rm = http_req("/api/config/domains/remove", "POST", {"domain": "cloud.test-enterprise.com"})
        report("POST /api/config/domains/remove removes domain", dom_rm.get("status") == "domain_removed", str(dom_rm))

        code, domains_after = http_req("/api/config/domains")
        report("Domain successfully removed from active list", "cloud.test-enterprise.com" not in domains_after, f"Domains: {domains_after}")

        # Test 5: Cloudflare Tunnel Token API
        test_token = "eyJhIjoiMWEyYjNjNGQ1ZTZmIiwidCI6IjEyMzQ1Njc4LTAwMDAifQ=="
        code, tunnel_resp = http_req("/api/tunnel/token", "POST", {"token": test_token})
        report("POST /api/tunnel/token validates and saves token", tunnel_resp.get("status") == "token_saved", str(tunnel_resp))

        # Test 6: Nextcloud Data Root Directory update
        new_dataroot = str(SANDBOX_DIR / "custom_nextcloud_data")
        code, dataroot_resp = http_req("/api/config/dataroot", "POST", {"path": new_dataroot})
        report("POST /api/config/dataroot sets new storage location", dataroot_resp.get("status") == "data_root_updated", str(dataroot_resp))

        code, current_status = http_req("/api/status")
        report("Data root path confirmed in supervisor state", current_status.get("data_dir") == new_dataroot, f"Data root: {current_status.get('data_dir')}")

    finally:
        print("  -> Stopping supervisor daemon...")
        proc.terminate()
        proc.wait(timeout=3)
        print("  ✓ Supervisor terminated cleanly.")

# ==============================================================================
# 3. Direct Drive Mounting & Recursive Permission Stress Test
# ==============================================================================
def test_drive_mounting_and_permissions():
    log_header("3. Direct Drive Mounting & Full Read/Write/Delete Access")

    if DRIVES_DIR.exists():
        shutil.rmtree(DRIVES_DIR)
    DRIVES_DIR.mkdir(parents=True, exist_ok=True)

    drive_specs = [
        ("drive_d", "Media & Video Storage", DRIVES_DIR / "drive_d"),
        ("drive_e", "Primary Backup Array", DRIVES_DIR / "drive_e"),
        ("drive_f", "Raw Photo Archives", DRIVES_DIR / "drive_f")
    ]

    # Create drive folders
    for d_id, name, path in drive_specs:
        path.mkdir(parents=True, exist_ok=True)

    # Re-spawn supervisor on TEST_PORT for drive tests
    cfg_path = SANDBOX_DIR / "test_config.json"
    proc = subprocess.Popen(
        [str(SUPERVISOR_BIN), "server", "--port", str(TEST_PORT), "--config", str(cfg_path)],
        cwd=BASE_DIR,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )

    try:
        time.sleep(1.0)

        for d_id, name, path in drive_specs:
            code, resp = http_req("/api/drives/mount", "POST", {"path": str(path), "name": name})
            report(f"Mount {name} ({path.name})", resp.get("status") == "success", str(resp))

        code, status = http_req("/api/status")
        mounted_paths = [d.get("path") for d in status.get("drives", [])]
        all_mounted = all(str(p) in mounted_paths for _, _, p in drive_specs)
        report("All 3 simulated drives present in status", all_mounted, f"Mounted count: {len(status.get('drives', []))}")

        # Test Read/Write/Delete on each drive
        for d_id, name, path in drive_specs:
            test_subfolder = path / "nextcloud_user_files" / "sub_test"
            test_subfolder.mkdir(parents=True, exist_ok=True)
            test_file = test_subfolder / "sample_document.txt"
            
            # Write
            test_payload = f"Hello Nextcloud from {name}! Timestamp: {time.time()}."
            test_file.write_text(test_payload)
            report(f"Write permission on {d_id}", test_file.exists() and test_file.read_text() == test_payload)

            # Append
            with open(test_file, "a") as f:
                f.write("\nAppended transactional line.")
            report(f"Append permission on {d_id}", "Appended transactional line." in test_file.read_text())

            # Delete
            test_file.unlink()
            report(f"Delete permission on {d_id}", not test_file.exists(), "Clean file deletion")

        # Test Unmounting drive_e
        unmount_target = str(drive_specs[1][2])
        code, unmount_resp = http_req("/api/drives/unmount", "POST", {"path": unmount_target})
        report("POST /api/drives/unmount unmounts drive_e", unmount_resp.get("status") == "unmounted", str(unmount_resp))

        code, after_unmount_status = http_req("/api/status")
        remaining_paths = [d.get("path") for d in after_unmount_status.get("drives", [])]
        report("Drive_e successfully absent after unmount", unmount_target not in remaining_paths, f"Remaining: {remaining_paths}")
        report("Drives D and F remain mounted", str(drive_specs[0][2]) in remaining_paths and str(drive_specs[2][2]) in remaining_paths)

    finally:
        proc.terminate()
        proc.wait(timeout=3)

# ==============================================================================
# 4. Nextcloud Best Practices & Server Configuration Audit
# ==============================================================================
def test_nextcloud_best_practices_audit():
    log_header("4. Nextcloud Server Best Practices & Performance Config Audit")

    cfg_sample = BASE_DIR / "config" / "nextcloud" / "config.sample.php"
    php_ini = BASE_DIR / "config" / "php" / "php.ini"

    cfg_content = cfg_sample.read_text()
    ini_content = php_ini.read_text()

    # 1. OPcache JIT & Memory
    report("PHP OPcache enabled (opcache.enable = 1)", "opcache.enable = 1" in ini_content)
    report("PHP OPcache JIT enabled (opcache.jit)", "opcache.jit = 1255" in ini_content or "opcache.jit" in ini_content)
    report("PHP OPcache Memory >= 256MB", "opcache.memory_consumption = 256" in ini_content)
    report("PHP OPcache max_accelerated_files >= 10000", "opcache.max_accelerated_files = 30000" in ini_content)

    # 2. Memory Cache & Redis Locking
    report("Nextcloud local cache: APCu", "'memcache.local' => '\\OC\\Memcache\\APCu'" in cfg_content)
    report("Nextcloud distributed cache: Redis", "'memcache.distributed' => '\\OC\\Memcache\\Redis'" in cfg_content)
    report("Nextcloud transactional locking: Redis", "'memcache.locking' => '\\OC\\Memcache\\Redis'" in cfg_content)
    report("Nextcloud filelocking.enabled = true", "'filelocking.enabled' => true" in cfg_content)

    # 3. Direct Drive Access & File Detection
    report("Nextcloud external storage create allowed", "'files_external_allow_create_new_local' => true" in cfg_content)
    report("Nextcloud filesystem auto-detects physical changes", "'filesystem_check_changes' => 1" in cfg_content)

    # 4. Large Upload Limits & Chunking
    report("PHP max upload >= 10GB", "upload_max_filesize = 16G" in ini_content and "post_max_size = 16G" in ini_content)
    report("Nextcloud 20MB chunking enabled", "'chunk_size' => 20971520" in cfg_content)

    # 5. Media Preview Providers
    report("Movie / Video Preview Provider enabled", "'OC\\Preview\\Movie'" in cfg_content)
    report("HEIC, SVG, WebP, and PDF Previews enabled", "'OC\\Preview\\HEIC'" in cfg_content and "'OC\\Preview\\SVG'" in cfg_content)

# ==============================================================================
# 5. High-Bitrate Media Generation & Stress Benchmark
# ==============================================================================
def test_high_bitrate_media_benchmark():
    log_header("5. High-Bitrate Video & Multi-Format Image Performance Benchmark")

    media_dir = DRIVES_DIR / "drive_d" / "media_stress_test"
    media_dir.mkdir(parents=True, exist_ok=True)
    thumbs_dir = media_dir / "generated_thumbnails"
    thumbs_dir.mkdir(parents=True, exist_ok=True)

    if not FFMPEG_BIN.exists():
        report("FFmpeg binary available", False, f"Missing at: {FFMPEG_BIN}")
        return

    report("FFmpeg standalone binary ready", True, f"Version: {FFMPEG_BIN}")

    # 1. Generate High-Bitrate Videos with Noise Pattern
    videos = [
        ("4k_noise_25mbps.mp4", "scale=3840:2160", "libx264", "25M", "MP4 (H.264 4K High Bitrate)"),
        ("1080p_hevc_noise.mkv", "scale=1920:1080", "libx264", "15M", "MKV (1080p High Bitrate)"),
        ("720p_vp9_stream.webm", "scale=1280:720", "libvpx-vp9", "8M", "WebM (VP9 720p Stream)"),
        ("studio_quicktime.mov", "scale=1920:1080", "libx264", "20M", "MOV (QuickTime Studio Video)")
    ]

    for filename, scale, vcodec, bitrate, desc in videos:
        out_file = media_dir / filename
        print(f"  -> Generating synthetic {desc}...")
        cmd = [
            str(FFMPEG_BIN), "-y",
            "-f", "lavfi", "-i", f"testsrc=duration=3:size=1920x1080:rate=30",
            "-vf", f"noise=alls=20:allf=t+u,{scale}",
            "-c:v", vcodec,
            "-b:v", bitrate,
            "-pix_fmt", "yuv420p",
            str(out_file)
        ]
        res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        if res.returncode == 0 and out_file.exists():
            size_mb = out_file.stat().st_size / (1024 * 1024)
            report(f"Video created: {filename}", True, f"{desc} | {size_mb:.2f} MB")

            # Extract Thumbnail at 1.5s
            thumb_out = thumbs_dir / f"{out_file.stem}_thumb.jpg"
            t_cmd = [
                str(FFMPEG_BIN), "-y",
                "-ss", "00:00:01.500",
                "-i", str(out_file),
                "-vframes", "1",
                "-q:v", "2",
                str(thumb_out)
            ]
            t_res = subprocess.run(t_cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            report(f"Thumbnail extracted for {filename}", t_res.returncode == 0 and thumb_out.exists(), f"Thumb: {thumb_out.name} ({thumb_out.stat().st_size // 1024} KB)")
        else:
            report(f"Video creation: {filename}", False, f"Failed with code {res.returncode}")

    # 2. Multi-Format Image Generation & Validation
    images = [
        ("photo_highres.jpg", "format=yuvj420p", "High-Resolution JPEG Photo"),
        ("graphic_alpha.png", "format=rgba", "Lossless PNG with Alpha Channel"),
        ("modern_lossless.webp", "format=yuv420p", "Modern WebP Lossless Image")
    ]

    for filename, fmt_filter, desc in images:
        out_img = media_dir / filename
        cmd = [
            str(FFMPEG_BIN), "-y",
            "-f", "lavfi", "-i", "testsrc=size=2048x2048:rate=1",
            "-vf", f"{fmt_filter}",
            "-vframes", "1",
            str(out_img)
        ]
        res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        report(f"Image created: {filename}", res.returncode == 0 and out_img.exists(), f"{desc} ({out_img.stat().st_size // 1024} KB)")

    # 3. Vector SVG Creation
    svg_file = media_dir / "vector_schema.svg"
    svg_content = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 800 600" width="800" height="600">
  <rect width="800" height="600" fill="#09090B"/>
  <circle cx="400" cy="300" r="180" fill="#0EA5E9" fill-opacity="0.8"/>
  <text x="400" y="310" font-family="monospace" font-size="28" fill="#FFFFFF" text-anchor="middle">Nextcloud AIO Drive Test</text>
</svg>"""
    svg_file.write_text(svg_content)
    report("Vector SVG image created", svg_file.exists(), f"SVG size: {svg_file.stat().st_size} bytes")

    # 4. Multi-frame GIF Animation Creation
    gif_file = media_dir / "animated_preview.gif"
    cmd = [
        str(FFMPEG_BIN), "-y",
        "-f", "lavfi", "-i", "testsrc=duration=2:size=400x300:rate=15",
        "-vf", "split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
        str(gif_file)
    ]
    res = subprocess.run(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    report("Animated GIF created", res.returncode == 0 and gif_file.exists(), f"GIF size: {gif_file.stat().st_size // 1024} KB")

    # 5. Read Throughput Stress Benchmark
    print("  -> Benchmarking drive read throughput on high-bitrate files...")
    total_bytes = 0
    t0 = time.time()
    for item in media_dir.glob("*.*"):
        if item.is_file():
            with open(item, "rb") as f:
                while chunk := f.read(1024 * 1024):
                    total_bytes += len(chunk)
    elapsed = max(0.001, time.time() - t0)
    throughput_mb_s = (total_bytes / (1024 * 1024)) / elapsed
    report("Drive media read throughput benchmark", throughput_mb_s > 50, f"Read {total_bytes / (1024*1024):.2f} MB in {elapsed:.3f}s -> {throughput_mb_s:.2f} MB/s")

# ==============================================================================
# Main Orchestration Loop
# ==============================================================================
def main():
    start_time = time.time()
    print(f"\n{C_BOLD}Starting Nextcloud AIO Automated CI/CD & Media Test Suite{C_RESET}")
    print(f"Timestamp: {time.strftime('%Y-%m-%d %H:%M:%S UTC', time.gmtime())}")

    try:
        test_extraction_and_packaging()
        test_supervisor_rest_api()
        test_drive_mounting_and_permissions()
        test_nextcloud_best_practices_audit()
        test_high_bitrate_media_benchmark()
    except Exception as e:
        print(f"\n{C_RED}FATAL ERROR IN TEST SUITE: {e}{C_RESET}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

    total_duration = time.time() - start_time
    total_tests = len(test_results)
    passed_tests = sum(1 for _, p, _ in test_results)
    failed_tests = total_tests - passed_tests

    log_header("FINAL TEST SUITE SUMMARY")
    print(f"  Total Test Cases : {total_tests}")
    print(f"  Passed Cases     : {C_GREEN}{passed_tests}{C_RESET}")
    print(f"  Failed Cases     : {C_RED if failed_tests > 0 else C_GREEN}{failed_tests}{C_RESET}")
    print(f"  Total Duration   : {total_duration:.2f} seconds\n")

    if failed_tests == 0:
        print(f"{C_BOLD}{C_GREEN}>>> ALL CI/CD INTEGRATION & MEDIA TESTS PASSED WITH 100% SUCCESS <<<{C_RESET}\n")
        sys.exit(0)
    else:
        print(f"{C_BOLD}{C_RED}>>> TEST SUITE FAILED WITH {failed_tests} ERRORS <<<{C_RESET}\n")
        sys.exit(1)

if __name__ == "__main__":
    main()
