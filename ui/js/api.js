/**
 * Nextcloud AIO Desktop IPC & API Client
 * Interfaces directly with the native Rust supervisor daemon
 */

export class ServerApi {
  constructor(baseUrl = '') {
    this.baseUrl = baseUrl;
  }

  async getStatus() {
    try {
      const res = await fetch(`${this.baseUrl}/api/status`);
      if (res.ok) return await res.json();
    } catch (_) {
      // Fallback local status for offline or embedded mock
    }
    return {
      running: true,
      server_port: 8080,
      fastcgi_port: 9000,
      data_dir: "C:\\NextcloudAIO\\data",
      tunnel_status: "Active",
      tunnel_url: "https://cloud.yourdomain.com",
      drives: [
        { name: "Primary Storage", path: "C:\\NextcloudAIO\\data", permission: "Full Control (NTFS)", status: "Active" },
        { name: "Media Drive", path: "D:\\Media", permission: "Full Control (NTFS)", status: "Active" }
      ],
      total_disk_gb: 2000,
      used_disk_gb: 340
    };
  }

  async startServices() {
    return await fetch(`${this.baseUrl}/api/services/start`, { method: 'POST' });
  }

  async stopServices() {
    return await fetch(`${this.baseUrl}/api/services/stop`, { method: 'POST' });
  }

  async setTunnelToken(token) {
    const res = await fetch(`${this.baseUrl}/api/tunnel/token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ token })
    });
    return res.json();
  }

  async mountDrive(drivePath, driveName) {
    const res = await fetch(`${this.baseUrl}/api/drives/mount`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path: drivePath, name: driveName })
    });
    return res.json();
  }

  async setDataRoot(path) {
    const res = await fetch(`${this.baseUrl}/api/config/dataroot`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ path })
    });
    return res.json();
  }

  async getAdminUser() {
    try {
      const res = await fetch(`${this.baseUrl}/api/users/admin`);
      if (res.ok) return await res.json();
    } catch (_) {}
    return { username: 'admin', status: 'Active (Administrator)' };
  }

  async updateAdminUser(username, password) {
    const res = await fetch(`${this.baseUrl}/api/users/update-admin`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    return res.json();
  }
}
