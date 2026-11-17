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
    this.initSplashScreen();
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

        const titleElem = document.getElementById('current-view-title');
        if (titleElem) {
          titleElem.textContent = item.querySelector('span')?.textContent || 'Dashboard';
        }
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
    // State-aware Server Toggle Button
    const btnToggle = document.getElementById('btn-toggle-server');
    if (btnToggle) {
      btnToggle.addEventListener('click', async () => {
        btnToggle.disabled = true;
        const isRunning = this.state?.running ?? true;

        if (isRunning) {
          this.logViewer.append('CORE', 'Sending SIGTERM to child processes...');
          await this.api.stopServices();
          this.logViewer.append('CORE', 'All services stopped gracefully.');
        } else {
          this.logViewer.append('CORE', 'Starting server stack (Caddy, PHP-CGI, Cloudflared)...');
          await this.api.startServices();
          this.logViewer.append('CORE', 'Services started successfully.');
        }

        btnToggle.disabled = false;
        await this.refresh();
      });
    }

    // Retain hidden legacy start/stop buttons for full backwards compatibility
    const btnStart = document.getElementById('btn-start-services');
    if (btnStart) {
      btnStart.addEventListener('click', async () => {
        this.logViewer.append('CORE', 'Starting server stack (Caddy, PHP-CGI, Cloudflared)...');
        await this.api.startServices();
        this.logViewer.append('CORE', 'Services started successfully.');
        await this.refresh();
      });
    }

    const btnStop = document.getElementById('btn-stop-services');
    if (btnStop) {
      btnStop.addEventListener('click', async () => {
        this.logViewer.append('CORE', 'Sending SIGTERM to child processes...');
        await this.api.stopServices();
        this.logViewer.append('CORE', 'All services stopped gracefully.');
        await this.refresh();
      });
    }

    // Copy endpoint buttons
    document.querySelectorAll('.btn-copy-icon').forEach(btn => {
      btn.addEventListener('click', (e) => {
        e.preventDefault();
        const copyVal = btn.getAttribute('data-copy');
        if (copyVal) {
          navigator.clipboard.writeText(copyVal);
          const origHtml = btn.innerHTML;
          btn.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#10B981" stroke-width="2"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
          setTimeout(() => { btn.innerHTML = origHtml; }, 1500);
        }
      });
    });

    // Browse Folder for Drive Form
    const btnBrowseDrive = document.getElementById('btn-browse-drive');
    if (btnBrowseDrive) {
      btnBrowseDrive.addEventListener('click', () => {
        const pathInput = document.getElementById('input-drive-path');
        const chosen = prompt("Enter drive letter or absolute path to mount (e.g. D:\\ or E:\\Storage):", pathInput.value || "D:\\");
        if (chosen && chosen.trim()) {
          pathInput.value = chosen.trim();
        }
      });
    }

    // Add Drive On-Demand Form
    const formAddDrive = document.getElementById('form-add-drive');
    if (formAddDrive) {
      formAddDrive.addEventListener('submit', async (e) => {
        e.preventDefault();
        const drivePath = document.getElementById('input-drive-path').value.trim();
        const driveName = document.getElementById('input-drive-name').value.trim() || "External Storage";

        if (!drivePath) return;

        this.logViewer.append('CORE', `Granting NTFS universal ACLs on '${drivePath}'...`);
        this.logViewer.append('CORE', `icacls "${drivePath}" /grant *S-1-1-0:(OI)(CI)F /T /C /Q`);

        try {
          await this.api.mountDrive(drivePath, driveName);
          this.logViewer.append('CORE', `Drive '${driveName}' (${drivePath}) mounted successfully.`);
          document.getElementById('input-drive-path').value = '';
          document.getElementById('input-drive-name').value = '';
          await this.refresh();
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
        document.getElementById('input-tunnel-token').value = '';
        await this.refresh();
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
          await this.refresh();
        }
      });
    }

    // Server Web Port Form
    const formPort = document.getElementById('form-change-port');
    if (formPort) {
      formPort.addEventListener('submit', async (e) => {
        e.preventDefault();
        const p = document.getElementById('input-server-port').value.trim();
        if (p) {
          this.logViewer.append('CORE', `Configuring HTTP server port to ${p}...`);
          await this.api.setPort(p);
          this.logViewer.append('CORE', `Server port set to ${p}.`);
          await this.refresh();
        }
      });
    }

    // Trusted Domains Form
    const formDomain = document.getElementById('form-add-domain');
    if (formDomain) {
      formDomain.addEventListener('submit', async (e) => {
        e.preventDefault();
        const domain = document.getElementById('input-new-domain').value.trim();
        if (domain) {
          this.logViewer.append('CORE', `Adding trusted domain: ${domain}...`);
          await this.api.addTrustedDomain(domain);
          this.logViewer.append('CORE', `Trusted domain '${domain}' added.`);
          document.getElementById('input-new-domain').value = '';
          this.refreshDomains();
        }
      });
    }

    // In-App Admin User Management Form
    const formUpdateAdmin = document.getElementById('form-update-admin');
    if (formUpdateAdmin) {
      formUpdateAdmin.addEventListener('submit', async (e) => {
        e.preventDefault();
        const user = document.getElementById('lbl-admin-username')?.textContent || 'admin';
        const pass = document.getElementById('input-admin-password').value.trim();
        if (pass) {
          this.logViewer.append('CORE', `Updating administrator credentials for '${user}'...`);
          await this.api.updateAdminUser(user, pass);
          this.logViewer.append('CORE', `Administrator credentials updated successfully.`);
          document.getElementById('input-admin-password').value = '';
          await this.refresh();
        }
      });
    }
  }

  initSplashScreen() {
    const splash = document.getElementById('splash-screen');
    const splashLogoContainer = document.getElementById('splash-logo-container');
    const targetBrand = document.querySelector('.sidebar .brand-icon');

    if (!splash || !targetBrand) return;

    // Smoothly transition logo into the sidebar position after initial load
    setTimeout(() => {
      const targetRect = targetBrand.getBoundingClientRect();
      const sourceRect = splashLogoContainer.getBoundingClientRect();

      const deltaX = targetRect.left + (targetRect.width / 2) - (sourceRect.left + (sourceRect.width / 2));
      const deltaY = targetRect.top + (targetRect.height / 2) - (sourceRect.top + (sourceRect.height / 2));
      const scale = targetRect.width / sourceRect.width;

      splashLogoContainer.style.transform = `translate(${deltaX}px, ${deltaY}px) scale(${scale})`;
      splash.classList.add('splash-leaving');

      setTimeout(() => {
        splash.style.display = 'none';
      }, 850);
    }, 850);
  }

  async refresh() {
    this.state = await this.api.getStatus();
    const adminData = await this.api.getAdminUser();

    const isRunning = this.state?.running ?? true;
    const port = this.state?.server_port || 8080;
    const localUrl = `http://localhost:${port}`;

    // Update Header Status Indicator & State-aware Toggle Button
    const headerStatusDot = document.getElementById('header-status-dot');
    const headerStatusText = document.getElementById('header-status-text');
    const btnToggle = document.getElementById('btn-toggle-server');
    const iconToggle = document.getElementById('icon-toggle-server');
    const lblToggle = document.getElementById('lbl-toggle-server');
    const btnOpenNextcloud = document.getElementById('btn-open-nextcloud');

    if (isRunning) {
      if (headerStatusDot) {
        headerStatusDot.className = 'status-dot dot-running';
      }
      if (headerStatusText) {
        headerStatusText.textContent = 'Running';
      }
      if (btnToggle) {
        btnToggle.className = 'btn btn-destructive-outline';
      }
      if (lblToggle) {
        lblToggle.textContent = 'Stop Server';
      }
      if (iconToggle) {
        iconToggle.innerHTML = '<rect x="6" y="6" width="12" height="12"></rect>';
      }
      if (btnOpenNextcloud) {
        btnOpenNextcloud.href = localUrl;
        btnOpenNextcloud.className = 'btn btn-primary';
        btnOpenNextcloud.removeAttribute('disabled');
      }
    } else {
      if (headerStatusDot) {
        headerStatusDot.className = 'status-dot dot-stopped';
      }
      if (headerStatusText) {
        headerStatusText.textContent = 'Stopped';
      }
      if (btnToggle) {
        btnToggle.className = 'btn btn-success';
      }
      if (lblToggle) {
        lblToggle.textContent = 'Start Server';
      }
      if (iconToggle) {
        iconToggle.innerHTML = '<polygon points="5 3 19 12 5 21 5 3"></polygon>';
      }
      if (btnOpenNextcloud) {
        btnOpenNextcloud.href = '#';
        btnOpenNextcloud.className = 'btn btn-primary disabled';
        btnOpenNextcloud.setAttribute('disabled', 'true');
      }
    }

    // Update Dashboard Endpoints
    const linkCore = document.getElementById('link-core-endpoint');
    if (linkCore) {
      linkCore.href = localUrl;
      linkCore.textContent = localUrl;
      const copyBtn = linkCore.parentElement?.querySelector('.btn-copy-icon');
      if (copyBtn) copyBtn.setAttribute('data-copy', localUrl);
    }

    const lblCaddyPort = document.getElementById('lbl-caddy-port');
    if (lblCaddyPort) {
      lblCaddyPort.textContent = `${port} (Proxy to FastCGI)`;
    }

    // Update Admin label
    const adminUserElem = document.getElementById('lbl-admin-username');
    if (adminUserElem && adminData.username) {
      adminUserElem.textContent = adminData.username;
    }

    // Update Port Field
    const inputPort = document.getElementById('input-server-port');
    if (inputPort && this.state.server_port) {
      inputPort.value = this.state.server_port;
    }

    // Update Tunnel displays
    const tunnelUrl = this.state.tunnel_url || 'https://cloud.yourdomain.com';
    const tunnelUrlElem = document.getElementById('lbl-tunnel-url');
    if (tunnelUrlElem) {
      tunnelUrlElem.href = tunnelUrl;
      tunnelUrlElem.textContent = tunnelUrl;
      const copyTunnelBtn = document.getElementById('btn-copy-tunnel');
      if (copyTunnelBtn) copyTunnelBtn.setAttribute('data-copy', tunnelUrl);
    }

    const cfgTunnelUrl = document.getElementById('lbl-tunnel-active-url');
    if (cfgTunnelUrl) {
      cfgTunnelUrl.href = tunnelUrl;
      cfgTunnelUrl.textContent = tunnelUrl;
    }

    // Render storage gauge
    this.gauge.render(this.state);

    // Update Data Directory label
    const dataDirElem = document.getElementById('lbl-data-dir');
    if (dataDirElem && this.state.data_dir) {
      dataDirElem.textContent = this.state.data_dir;
    }

    // Render Trusted Domains list
    this.refreshDomains();

    // Render Drive Cards with single subtle metadata label and unmount on hover
    const driveContainer = document.getElementById('drive-list-container');
    if (driveContainer && this.state.drives) {
      driveContainer.innerHTML = this.state.drives.map(d => {
        const total = d.total_gb || 1000;
        const used = d.used_gb || 200;
        const free = d.free_gb || (total - used);
        const pct = Math.min(100, Math.round((used / total) * 100));
        return `
        <div class="drive-card">
          <div class="drive-card-header">
            <div class="drive-info">
              <div class="drive-icon">
                <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                </svg>
              </div>
              <div>
                <div class="drive-name">${d.name}</div>
                <div class="drive-path">${d.path}</div>
              </div>
            </div>
            <div style="display:flex; align-items:center; gap:12px;">
              <span class="drive-meta-label">NTFS • Mounted</span>
              <button type="button" class="btn-unmount-drive" data-path="${d.path}" title="Unmount external drive">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
                Unmount
              </button>
            </div>
          </div>
          <div style="margin-top:8px;">
            <div style="display:flex; justify-content:space-between; font-size:0.75rem; color:var(--text-secondary); margin-bottom:4px; font-family:var(--font-mono);">
              <span>${free} GB free of ${total} GB</span>
              <span>${pct}% used</span>
            </div>
            <div class="storage-bar-track" style="margin:0;">
              <div class="storage-bar-fill" style="width: ${pct}%;"></div>
            </div>
          </div>
        </div>
      `;}).join('');

      // Bind unmount click handlers
      driveContainer.querySelectorAll('.btn-unmount-drive').forEach(btn => {
        btn.addEventListener('click', async (e) => {
          e.stopPropagation();
          const drivePath = btn.getAttribute('data-path');
          if (confirm(`Unmount external drive '${drivePath}'?`)) {
            this.logViewer.append('CORE', `Unmounting external drive: ${drivePath}...`);
            await this.api.unmountDrive(drivePath);
            this.logViewer.append('CORE', `Drive '${drivePath}' unmounted.`);
            await this.refresh();
          }
        });
      });
    }
  }

  async refreshDomains() {
    const listElem = document.getElementById('trusted-domains-list');
    if (!listElem) return;
    const domains = await this.api.getTrustedDomains();
    const customDomains = (domains || []).filter(d => d !== 'localhost' && d !== '127.0.0.1' && d !== '::1');

    if (customDomains.length === 0) {
      listElem.innerHTML = `<span style="font-size:0.76rem; color:var(--text-muted); font-family:var(--font-mono);">No custom domains configured.</span>`;
      return;
    }

    listElem.innerHTML = customDomains.map(d => `
      <span class="mono-badge" style="display:inline-flex; align-items:center; gap:6px; padding:3px 8px; font-size:0.78rem;">
        <span>${d}</span>
        <button type="button" data-domain="${d}" class="btn-remove-domain" title="Remove domain" style="background:none; border:none; color:var(--text-muted); cursor:pointer; font-size:0.95rem; line-height:1; padding:0 2px;">×</button>
      </span>
    `).join('');

    listElem.querySelectorAll('.btn-remove-domain').forEach(btn => {
      btn.addEventListener('click', async () => {
        const dom = btn.getAttribute('data-domain');
        await this.api.removeTrustedDomain(dom);
        this.logViewer.append('CORE', `Removed trusted domain: ${dom}`);
        this.refreshDomains();
      });
    });
  }
}

// Bootstrap application on DOM ready
document.addEventListener('DOMContentLoaded', () => {
  window.nextcloudApp = new AppController();
});
