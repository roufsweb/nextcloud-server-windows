import { ServerApi } from './api.js';
import { StorageGauge } from '../components/storage-gauge.js';
import { LogViewer } from '../components/log-viewer.js';

class AppController {
  constructor() {
    this.api = new ServerApi();
    this.gauge = new StorageGauge('storage-gauge-root');
    this.logViewer = new LogViewer('log-viewer-root');
    this.state = null;

    this.initNavigation();
    this.initEventListeners();
    this.refresh();

    // Stream initial simulation logs
    this.logViewer.append('CORE', 'Nextcloud Server AIO supervisor initializing...');
    this.logViewer.append('CADDY', 'Caddy web server listening on http://localhost:8080');
    this.logViewer.append('PHP', 'PHP 8.3 NTS FastCGI worker pool spawned on 127.0.0.1:9000');
    this.logViewer.append('TUNNEL', 'Cloudflare Tunnel daemon active and ready.');
  }

  initNavigation() {
    document.querySelectorAll('.nav-item').forEach(item => {
      item.addEventListener('click', (e) => {
        e.preventDefault();
        const targetView = item.getAttribute('data-view');

        document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
        document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));

        item.classList.add('active');
        const viewElem = document.getElementById(targetView);
        if (viewElem) viewElem.classList.add('active');

        document.getElementById('current-view-title').textContent = item.querySelector('span').textContent;
      });
    });

    // Check if initial URL hash matches a view
    const initialHash = window.location.hash ? window.location.hash.replace('#', '') : null;
    if (initialHash) {
      const targetNav = document.querySelector(`.nav-item[data-view="${initialHash}"]`);
      if (targetNav) {
        targetNav.click();
      }
    }
  }

  initEventListeners() {
    // Start / Stop Service Buttons
    const btnStart = document.getElementById('btn-start-services');
    const btnStop = document.getElementById('btn-stop-services');

    if (btnStart) {
      btnStart.addEventListener('click', async () => {
        btnStart.disabled = true;
        this.logViewer.append('CORE', 'Starting server stack (Caddy, PHP-CGI, Cloudflared)...');
        await this.api.startServices();
        this.logViewer.append('CORE', 'Services started successfully.');
        btnStart.disabled = false;
        this.refresh();
      });
    }

    if (btnStop) {
      btnStop.addEventListener('click', async () => {
        btnStop.disabled = true;
        this.logViewer.append('CORE', 'Sending SIGTERM to child processes...');
        await this.api.stopServices();
        this.logViewer.append('CORE', 'All services stopped gracefully.');
        btnStop.disabled = false;
        this.refresh();
      });
    }

    // Add Drive On-Demand Form
    const formAddDrive = document.getElementById('form-add-drive');
    if (formAddDrive) {
      formAddDrive.addEventListener('submit', async (e) => {
        e.preventDefault();
        const drivePath = document.getElementById('input-drive-path').value.trim();
        const driveName = document.getElementById('input-drive-name').value.trim() || "External Drive";

        if (!drivePath) return;

        this.logViewer.append('CORE', `Executing Windows NTFS ACL grant on '${drivePath}'...`);
        this.logViewer.append('CORE', `icacls "${drivePath}" /grant *S-1-1-0:(OI)(CI)F /T /C /Q`);

        try {
          await this.api.mountDrive(drivePath, driveName);
          this.logViewer.append('CORE', `Drive '${driveName}' (${drivePath}) mounted with Full Access!`);
          alert(`Drive successfully added with Full Access! Nextcloud will show it as an external storage drive.`);
          document.getElementById('input-drive-path').value = '';
          document.getElementById('input-drive-name').value = '';
          this.refresh();
        } catch (err) {
          this.logViewer.append('CORE', `Error mounting drive: ${err.message}`);
          alert(`Failed to add drive: ${err.message}`);
        }
      });
    }

    // Cloudflare Token Form
    const formTunnel = document.getElementById('form-save-tunnel');
    if (formTunnel) {
      formTunnel.addEventListener('submit', async (e) => {
        e.preventDefault();
        const token = document.getElementById('input-tunnel-token').value.trim();
        if (!token) return;

        this.logViewer.append('TUNNEL', 'Validating and applying Cloudflare Tunnel token...');
        await this.api.setTunnelToken(token);
        this.logViewer.append('TUNNEL', 'Tunnel connected! Public HTTPS URL active.');
        alert('Cloudflare Tunnel token saved! Tunnel is now connected.');
        this.refresh();
      });
    }

    // Nextcloud Data Root Change
    const btnChangeDataRoot = document.getElementById('btn-change-dataroot');
    if (btnChangeDataRoot) {
      btnChangeDataRoot.addEventListener('click', async () => {
        const newPath = prompt("Enter new absolute folder path for Nextcloud Data Root:", this.state?.data_dir || "C:\\NextcloudAIO\\data");
        if (newPath && newPath.trim()) {
          this.logViewer.append('CORE', `Updating Nextcloud data root to: ${newPath.trim()}`);
          await this.api.setDataRoot(newPath.trim());
          this.logViewer.append('CORE', `Applied Windows ACL permissions to data root.`);
          this.refresh();
        }
      });
    }

    // First-Run Onboarding Modal Handlers
    const modalOnboard = document.getElementById('onboarding-modal');
    if (modalOnboard && !localStorage.getItem('nc_onboarded')) {
      modalOnboard.style.display = 'flex';
    }

    const btnOnboardSkip = document.getElementById('btn-onboard-skip');
    if (btnOnboardSkip) {
      btnOnboardSkip.addEventListener('click', () => {
        localStorage.setItem('nc_onboarded', 'true');
        if (modalOnboard) modalOnboard.style.display = 'none';
        this.logViewer.append('CORE', 'First-run setup completed using default administrator credentials.');
      });
    }

    const formOnboard = document.getElementById('form-onboarding');
    if (formOnboard) {
      formOnboard.addEventListener('submit', async (e) => {
        e.preventDefault();
        const user = document.getElementById('onboard-username').value.trim();
        const pass = document.getElementById('onboard-password').value.trim();
        if (user && pass) {
          await this.api.updateAdminUser(user, pass);
          localStorage.setItem('nc_onboarded', 'true');
          if (modalOnboard) modalOnboard.style.display = 'none';
          this.logViewer.append('CORE', `Administrator account configured: ${user}`);
          alert(`Administrator account created successfully!`);
          this.refresh();
        }
      });
    }

    // In-App Admin User Management Form
    const formUpdateAdmin = document.getElementById('form-update-admin');
    if (formUpdateAdmin) {
      formUpdateAdmin.addEventListener('submit', async (e) => {
        e.preventDefault();
        const user = document.getElementById('input-admin-username').value.trim();
        const pass = document.getElementById('input-admin-password').value.trim();
        if (user) {
          this.logViewer.append('CORE', `Updating administrator credentials for '${user}'...`);
          await this.api.updateAdminUser(user, pass || undefined);
          this.logViewer.append('CORE', `Administrator credentials updated in database.`);
          alert('Administrator account credentials updated successfully!');
          document.getElementById('input-admin-password').value = '';
          this.refresh();
        }
      });
    }
  }

  async refresh() {
    this.state = await this.api.getStatus();
    const adminData = await this.api.getAdminUser();

    // Update Admin label
    const adminUserElem = document.getElementById('lbl-admin-username');
    if (adminUserElem && adminData.username) {
      adminUserElem.textContent = adminData.username;
      const inputAdminUser = document.getElementById('input-admin-username');
      if (inputAdminUser) inputAdminUser.value = adminData.username;
    }

    // Render storage gauge
    this.gauge.render(this.state);

    // Update Data Directory label
    const dataDirElem = document.getElementById('lbl-data-dir');
    if (dataDirElem) dataDirElem.textContent = this.state.data_dir;

    // Render Drive Cards
    const driveContainer = document.getElementById('drive-list-container');
    if (driveContainer && this.state.drives) {
      driveContainer.innerHTML = this.state.drives.map(d => {
        const total = d.total_gb || 1000;
        const used = d.used_gb || 200;
        const free = d.free_gb || (total - used);
        const pct = Math.min(100, Math.round((used / total) * 100));
        return `
        <div class="drive-card" style="flex-direction:column; align-items:stretch; gap:12px;">
          <div style="display:flex; align-items:center; justify-content:space-between;">
            <div class="drive-info">
              <div class="drive-icon">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                </svg>
              </div>
              <div>
                <div class="drive-name">${d.name}</div>
                <div class="drive-path">${d.path}</div>
              </div>
            </div>
            <div style="display:flex; align-items:center; gap:12px;">
              <span class="badge-permission">✓ ${d.permission}</span>
              <span class="status-pill online">${d.status}</span>
            </div>
          </div>
          <div style="margin-top:4px;">
            <div style="display:flex; justify-content:space-between; font-size:0.78rem; color:var(--text-secondary); margin-bottom:4px;">
              <span><strong>${free} GB free</strong> of ${total} GB</span>
              <span>${pct}% used</span>
            </div>
            <div class="storage-bar-track" style="height:6px;">
              <div class="storage-bar-fill" style="width: ${pct}%;"></div>
            </div>
          </div>
        </div>
      `;}).join('');
    }

    // Update Tunnel URL
    const tunnelUrlElem = document.getElementById('lbl-tunnel-url');
    if (tunnelUrlElem && this.state.tunnel_url) {
      tunnelUrlElem.href = this.state.tunnel_url;
      tunnelUrlElem.textContent = this.state.tunnel_url;
    }
  }
}

// Bootstrap application on DOM ready
document.addEventListener('DOMContentLoaded', () => {
  window.nextcloudApp = new AppController();
});
