import { ServerApi } from './api.js';
import { StorageGauge } from '../components/storage-gauge.js';
import { LogViewer } from '../components/log-viewer.js';

class AppController {
  constructor() {
    this.api = new ServerApi();
    this.gauge = new StorageGauge('storage-gauge-root');
    this.logViewer = new LogViewer('log-viewer-root');
    this.state = null;
    this.selectedCmdIndex = 0;

    this.initNavigation();
    this.initEventListeners();
    this.initCommandPalette();
    this.initDiagnostics();
    this.initSplashScreen();
    this.refresh();

    // Periodic live status and log sync every 3.5s
    this.pollTimer = setInterval(() => {
      this.refresh();
    }, 3500);

    // Stream initial supervisor startup events
    this.logViewer.append('CORE', 'Nextcloud Server AIO supervisor initializing...');
    this.logViewer.append('CADDY', 'Caddy web server listening on http://localhost:8080');
    this.logViewer.append('PHP', 'PHP 8.3 NTS FastCGI worker pool spawned on 127.0.0.1:9000');
    this.logViewer.append('TUNNEL', 'Cloudflare Tunnel daemon active and ready.');
  }

  showToast(message, type = 'info') {
    const container = document.getElementById('toast-container');
    if (!container) return;

    const toast = document.createElement('div');
    toast.className = `toast toast-${type}`;
    const iconMap = {
      success: `<svg class="toast-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#10B981" stroke-width="2"><polyline points="20 6 9 17 4 12"></polyline></svg>`,
      error: `<svg class="toast-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#EF4444" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="15" y1="9" x2="9" y2="15"></line><line x1="9" y1="9" x2="15" y2="15"></line></svg>`,
      info: `<svg class="toast-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#0EA5E9" stroke-width="2"><circle cx="12" cy="12" r="10"></circle><line x1="12" y1="16" x2="12" y2="12"></line><line x1="12" y1="8" x2="12.01" y2="8"></line></svg>`
    };

    toast.innerHTML = `
      ${iconMap[type] || iconMap.info}
      <div class="toast-msg">${message}</div>
    `;
    container.appendChild(toast);

    setTimeout(() => {
      toast.classList.add('toast-leaving');
      setTimeout(() => { toast.remove(); }, 250);
    }, 3200);
  }

  switchToView(viewId) {
    document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));

    const navItem = document.querySelector(`.nav-item[data-view="${viewId}"]`);
    if (navItem) navItem.classList.add('active');

    const viewElem = document.getElementById(viewId);
    if (viewElem) viewElem.classList.add('active');

    const titleElem = document.getElementById('current-view-title');
    if (titleElem && navItem) {
      titleElem.textContent = navItem.querySelector('span')?.textContent || 'Dashboard';
    }
  }

  initNavigation() {
    document.querySelectorAll('.nav-item').forEach(item => {
      item.addEventListener('click', (e) => {
        e.preventDefault();
        const targetView = item.getAttribute('data-view');
        this.switchToView(targetView);
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

  initCommandPalette() {
    const modal = document.getElementById('command-palette-modal');
    const input = document.getElementById('cmd-palette-input');
    const list = document.getElementById('cmd-palette-list');
    const btnOpen = document.getElementById('btn-open-palette');

    if (!modal || !input || !list) return;

    this.commands = [
      { id: 'open-web', title: 'Open Nextcloud in Browser', subtitle: 'Launch local web interface (port 8080)', icon: 'globe', action: () => { window.open(this.state?.server_port ? `http://localhost:${this.state.server_port}` : 'http://localhost:8080', '_blank'); } },
      { id: 'toggle-srv', title: 'Toggle Server Stack', subtitle: 'Start or stop Caddy, PHP FastCGI and Cloudflared', icon: 'power', action: () => { document.getElementById('btn-toggle-server')?.click(); } },
      { id: 'open-data', title: 'Open Nextcloud Data in File Explorer', subtitle: 'Browse user files directly on Windows', icon: 'folder', action: async () => { await this.api.openFolder(this.state?.data_dir || 'C:\\NextcloudAIO\\data'); this.showToast('Opened data folder in Windows Explorer', 'success'); } },
      { id: 'nav-dashboard', title: 'Go to Dashboard', subtitle: 'System overview, status grid & storage metrics', icon: 'grid', action: () => { this.switchToView('view-dashboard'); } },
      { id: 'nav-storage', title: 'Go to Storage & Drives', subtitle: 'Mount external drives with NTFS permissions', icon: 'hard-drive', action: () => { this.switchToView('view-storage'); } },
      { id: 'nav-config', title: 'Go to System Config', subtitle: 'Configure Cloudflare tunnel, web port & domains', icon: 'settings', action: () => { this.switchToView('view-config'); } },
      { id: 'nav-logs', title: 'Go to In-App Logs', subtitle: 'Stream real-time server logs with filter tabs', icon: 'terminal', action: () => { this.switchToView('view-logs'); } },
      { id: 'db-optimize', title: 'Optimize Database (WAL Checkpoint & VACUUM)', subtitle: 'Checkpoint WAL journal and reclaim disk space', icon: 'database', action: () => { document.getElementById('btn-db-optimize')?.click(); } },
      { id: 'db-backup', title: 'Create Hot Database Snapshot', subtitle: 'Atomic point-in-time backup (VACUUM INTO)', icon: 'archive', action: () => { document.getElementById('btn-db-backup')?.click(); } },
      { id: 'doctor', title: 'Run Nextcloud Doctor (Diagnostics & Self-Healing)', subtitle: 'Inspect port conflicts, missing binaries, permissions & 1-click fixes', icon: 'activity', action: () => { this.openDiagnosticsModal(); } },
      { id: 'copy-local', title: 'Copy Local Web Address', subtitle: 'Copy http://localhost:8080 to clipboard', icon: 'copy', action: () => { navigator.clipboard.writeText(`http://localhost:${this.state?.server_port || 8080}`); this.showToast('Copied local URL to clipboard!', 'success'); } },
      { id: 'copy-tunnel', title: 'Copy Public Cloudflare URL', subtitle: 'Copy public HTTPS domain to clipboard', icon: 'link', action: () => { navigator.clipboard.writeText(this.state?.tunnel_url || 'https://cloud.yourdomain.com'); this.showToast('Copied public HTTPS URL to clipboard!', 'success'); } }
    ];

    const openModal = () => {
      modal.classList.add('open');
      input.value = '';
      this.selectedCmdIndex = 0;
      this.renderCommandList('');
      setTimeout(() => input.focus(), 50);
    };

    const closeModal = () => {
      modal.classList.remove('open');
    };

    if (btnOpen) btnOpen.addEventListener('click', openModal);

    modal.addEventListener('click', (e) => {
      if (e.target === modal) closeModal();
    });

    window.addEventListener('keydown', (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        if (modal.classList.contains('open')) closeModal(); else openModal();
      } else if (e.key === 'Escape' && modal.classList.contains('open')) {
        closeModal();
      } else if (modal.classList.contains('open')) {
        const items = list.querySelectorAll('.cmd-item');
        if (e.key === 'ArrowDown') {
          e.preventDefault();
          this.selectedCmdIndex = Math.min(items.length - 1, this.selectedCmdIndex + 1);
          this.highlightCmdItem(items);
        } else if (e.key === 'ArrowUp') {
          e.preventDefault();
          this.selectedCmdIndex = Math.max(0, this.selectedCmdIndex - 1);
          this.highlightCmdItem(items);
        } else if (e.key === 'Enter') {
          e.preventDefault();
          if (items[this.selectedCmdIndex]) {
            items[this.selectedCmdIndex].click();
          }
        }
      }
    });

    input.addEventListener('input', () => {
      this.selectedCmdIndex = 0;
      this.renderCommandList(input.value.toLowerCase().trim());
    });
  }

  highlightCmdItem(items) {
    items.forEach((item, idx) => {
      if (idx === this.selectedCmdIndex) {
        item.classList.add('selected');
        item.scrollIntoView({ block: 'nearest' });
      } else {
        item.classList.remove('selected');
      }
    });
  }

  renderCommandList(query) {
    const list = document.getElementById('cmd-palette-list');
    const modal = document.getElementById('command-palette-modal');
    if (!list) return;

    const filtered = this.commands.filter(c => 
      !query || c.title.toLowerCase().includes(query) || c.subtitle.toLowerCase().includes(query)
    );

    if (filtered.length === 0) {
      list.innerHTML = `<div style="padding: 16px; text-align: center; color: var(--text-muted); font-size: 0.82rem;">No commands match "${query}"</div>`;
      return;
    }

    list.innerHTML = filtered.map((c, idx) => `
      <div class="cmd-item ${idx === this.selectedCmdIndex ? 'selected' : ''}" data-cmd-id="${c.id}">
        <div class="cmd-item-left">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
          <div>
            <div style="color:var(--text-primary); font-size:0.83rem;">${c.title}</div>
            <div style="color:var(--text-muted); font-size:0.72rem; margin-top:1px;">${c.subtitle}</div>
          </div>
        </div>
        <div class="cmd-item-right">↵ Execute</div>
      </div>
    `).join('');

    list.querySelectorAll('.cmd-item').forEach((item, idx) => {
      item.addEventListener('click', () => {
        modal?.classList.remove('open');
        filtered[idx].action();
      });
      item.addEventListener('mouseenter', () => {
        this.selectedCmdIndex = idx;
        const all = list.querySelectorAll('.cmd-item');
        all.forEach((el, i) => el.classList.toggle('selected', i === idx));
      });
    });
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
          this.showToast('Nextcloud server stack stopped gracefully.', 'info');
        } else {
          this.logViewer.append('CORE', 'Starting server stack (Caddy, PHP-CGI, Cloudflared)...');
          await this.api.startServices();
          this.logViewer.append('CORE', 'Services started successfully.');
          this.showToast('Nextcloud server stack started successfully.', 'success');
        }

        btnToggle.disabled = false;
        await this.refresh();
      });
    }

    // Hero Card Action Buttons
    const heroBtnOpenFolder = document.getElementById('hero-btn-open-folder');
    if (heroBtnOpenFolder) {
      heroBtnOpenFolder.addEventListener('click', async () => {
        const path = this.state?.data_dir || "C:\\NextcloudAIO\\data";
        await this.api.openFolder(path);
        this.showToast('Opened Nextcloud data folder in Windows Explorer', 'success');
      });
    }

    const heroBtnBackup = document.getElementById('hero-btn-backup');
    if (heroBtnBackup) {
      heroBtnBackup.addEventListener('click', () => {
        document.getElementById('btn-db-backup')?.click();
      });
    }

    // Nextcloud Data Root Open in Explorer Button
    const btnOpenDataRoot = document.getElementById('btn-open-dataroot');
    if (btnOpenDataRoot) {
      btnOpenDataRoot.addEventListener('click', async () => {
        const path = this.state?.data_dir || "C:\\NextcloudAIO\\data";
        await this.api.openFolder(path);
        this.showToast('Opened data directory in Windows Explorer', 'success');
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
          this.showToast(`Copied to clipboard: ${copyVal}`, 'success');
          const origHtml = btn.innerHTML;
          btn.innerHTML = `<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="#10B981" stroke-width="2"><polyline points="20 6 9 17 4 12"></polyline></svg>`;
          setTimeout(() => { btn.innerHTML = origHtml; }, 1500);
        }
      });
    });

    // Browse Folder for Drive Form (Native Windows Directory Picker)
    const btnBrowseDrive = document.getElementById('btn-browse-drive');
    if (btnBrowseDrive) {
      btnBrowseDrive.addEventListener('click', async () => {
        btnBrowseDrive.classList.add('is-loading');
        try {
          const chosen = await this.api.pickFolder('Select Storage Drive or Directory to Mount');
          if (chosen) {
            const pathInput = document.getElementById('input-drive-path');
            const nameInput = document.getElementById('input-drive-name');
            if (pathInput) pathInput.value = chosen;
            if (nameInput && !nameInput.value) {
              const segs = chosen.split(/[\\/]/).filter(Boolean);
              nameInput.value = segs[segs.length - 1] || 'External Storage';
            }
            this.showToast(`Selected: ${chosen}`, 'info');
          }
        } finally {
          btnBrowseDrive.classList.remove('is-loading');
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
          this.showToast(`Drive '${driveName}' mounted with NTFS permissions!`, 'success');
          document.getElementById('input-drive-path').value = '';
          document.getElementById('input-drive-name').value = '';
          await this.refresh();
        } catch (err) {
          this.logViewer.append('CORE', `Error mounting drive: ${err.message}`);
          this.showToast(`Failed to add drive: ${err.message}`, 'error');
        }
      });
    }

    // Cloudflare Token Form & Disconnect
    const formTunnel = document.getElementById('form-save-tunnel');
    if (formTunnel) {
      formTunnel.addEventListener('submit', async (e) => {
        e.preventDefault();
        const token = document.getElementById('input-tunnel-token').value.trim();
        if (!token) return;

        this.logViewer.append('TUNNEL', 'Validating and applying Cloudflare Tunnel token...');
        await this.api.setTunnelToken(token);
        this.logViewer.append('TUNNEL', 'Tunnel connected! Public HTTPS URL active.');
        this.showToast('Cloudflare Tunnel token connected successfully!', 'success');
        document.getElementById('input-tunnel-token').value = '';
        await this.refresh();
      });
    }

    const btnDisconnectTunnel = document.getElementById('btn-disconnect-tunnel');
    if (btnDisconnectTunnel) {
      btnDisconnectTunnel.addEventListener('click', async () => {
        if (confirm("Disconnect Cloudflare Tunnel and revert to local network mode only?")) {
          btnDisconnectTunnel.classList.add('is-loading');
          try {
            await this.api.setTunnelToken('');
            this.logViewer.append('TUNNEL', 'Cloudflare Tunnel disconnected.');
            this.showToast('Tunnel disconnected. Server is now in local mode.', 'info');
            await this.refresh();
          } finally {
            btnDisconnectTunnel.classList.remove('is-loading');
          }
        }
      });
    }

    // Jump from Dashboard to Tunnel Config
    const btnJumpTunnel = document.getElementById('btn-jump-tunnel');
    if (btnJumpTunnel) {
      btnJumpTunnel.addEventListener('click', () => {
        this.switchToView('view-config');
        setTimeout(() => {
          document.getElementById('input-tunnel-token')?.focus();
        }, 150);
      });
    }

    // Nextcloud Data Root Change (Native Windows Folder Picker)
    const btnChangeDataRoot = document.getElementById('btn-change-dataroot');
    if (btnChangeDataRoot) {
      btnChangeDataRoot.addEventListener('click', async () => {
        btnChangeDataRoot.classList.add('is-loading');
        try {
          const newPath = await this.api.pickFolder('Select New Nextcloud Data Directory');
          if (newPath && newPath.trim() && newPath !== this.state?.data_dir) {
            this.logViewer.append('CORE', `Updating Nextcloud data root to: ${newPath.trim()}`);
            await this.api.setDataRoot(newPath.trim());
            this.logViewer.append('CORE', `Applied Windows ACL permissions to data root.`);
            this.showToast(`Data root updated to: ${newPath.trim()}`, 'success');
            await this.refresh();
          }
        } finally {
          btnChangeDataRoot.classList.remove('is-loading');
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
          this.showToast(`Server web port set to ${p}`, 'success');
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
          this.showToast(`Added trusted domain: ${domain}`, 'success');
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
          this.showToast(`Administrator credentials updated for '${user}'!`, 'success');
          document.getElementById('input-admin-password').value = '';
          await this.refresh();
        }
      });
    }

    // Database Optimization & Backup Listeners
    const btnDbOptimize = document.getElementById('btn-db-optimize');
    const dbActionStatus = document.getElementById('lbl-db-action-status');
    if (btnDbOptimize) {
      btnDbOptimize.addEventListener('click', async () => {
        btnDbOptimize.disabled = true;
        this.logViewer.append('CORE', 'Executing WAL checkpoint (TRUNCATE) and VACUUM...');
        try {
          const res = await this.api.optimizeDatabase();
          this.logViewer.append('CORE', `Database optimized. Status: ${res.status}, New Size: ${(res.new_size_bytes / 1024).toFixed(1)} KB.`);
          this.showToast('WAL Checkpoint & VACUUM completed successfully!', 'success');
          if (dbActionStatus) {
            dbActionStatus.style.display = 'block';
            dbActionStatus.textContent = `✓ Checkpoint & VACUUM completed. Current size: ${(res.new_size_bytes / (1024 * 1024)).toFixed(2)} MB.`;
            setTimeout(() => { dbActionStatus.style.display = 'none'; }, 6000);
          }
          await this.refresh();
        } catch (err) {
          this.logViewer.append('CORE', `Optimization error: ${err.message}`);
          this.showToast(`Optimization failed: ${err.message}`, 'error');
        } finally {
          btnDbOptimize.disabled = false;
        }
      });
    }

    const btnDbBackup = document.getElementById('btn-db-backup');
    if (btnDbBackup) {
      btnDbBackup.addEventListener('click', async () => {
        btnDbBackup.disabled = true;
        this.logViewer.append('CORE', 'Executing atomic point-in-time database snapshot (VACUUM INTO)...');
        try {
          const res = await this.api.backupDatabase();
          this.logViewer.append('CORE', `Hot backup created: ${res.backup_file} (${(res.size_bytes / 1024).toFixed(1)} KB).`);
          this.showToast(`Snapshot created: ${res.backup_file}`, 'success');
          if (dbActionStatus) {
            dbActionStatus.style.display = 'block';
            dbActionStatus.textContent = `✓ Snapshot saved: ${res.backup_file} (${(res.size_bytes / (1024 * 1024)).toFixed(2)} MB).`;
            setTimeout(() => { dbActionStatus.style.display = 'none'; }, 6000);
          }
        } catch (err) {
          this.logViewer.append('CORE', `Backup error: ${err.message}`);
          this.showToast(`Backup error: ${err.message}`, 'error');
        } finally {
          btnDbBackup.disabled = false;
        }
      });
    }

    // Windows Auto-Start on Boot Toggle
    const chkAutostart = document.getElementById('chk-autostart');
    if (chkAutostart) {
      chkAutostart.addEventListener('change', async () => {
        const enabled = chkAutostart.checked;
        this.logViewer.append('CORE', `${enabled ? 'Registering' : 'Unregistering'} Windows startup boot key...`);
        try {
          const res = await this.api.setAutostart(enabled);
          this.logViewer.append('CORE', `Auto-start on boot is now ${res.enabled ? 'ENABLED' : 'DISABLED'}.`);
          this.showToast(`Windows startup on boot is now ${res.enabled ? 'ENABLED' : 'DISABLED'}`, 'info');
          this.updateAutostartUI(res.enabled);
        } catch (err) {
          this.logViewer.append('CORE', `Auto-start error: ${err.message}`);
          this.showToast(`Auto-start update failed: ${err.message}`, 'error');
          chkAutostart.checked = !enabled;
        }
      });
    }
  }

  updateAutostartUI(enabled) {
    const chk = document.getElementById('chk-autostart');
    if (chk) chk.checked = Boolean(enabled);
    const dot = document.getElementById('dot-autostart-status');
    const lbl = document.getElementById('lbl-autostart-status');
    if (dot) {
      dot.className = enabled ? 'status-dot dot-running' : 'status-dot dot-stopped';
    }
    if (lbl) {
      lbl.textContent = enabled ? 'Start on Boot: Active' : 'Start on Boot: Off';
    }
  }

  initDiagnostics() {
    const modal = document.getElementById('diagnostics-modal');
    const btnOpen = document.getElementById('btn-open-diagnostics');
    const btnViewAll = document.getElementById('btn-view-all-diag');
    const btnClose = document.getElementById('btn-close-diagnostics');
    const btnRerun = document.getElementById('btn-rerun-diagnostics');

    if (btnOpen) btnOpen.addEventListener('click', () => this.openDiagnosticsModal());
    if (btnViewAll) btnViewAll.addEventListener('click', () => this.openDiagnosticsModal());
    if (btnClose) btnClose.addEventListener('click', () => this.closeDiagnosticsModal());
    if (btnRerun) btnRerun.addEventListener('click', () => this.runDiagnostics());

    if (modal) {
      modal.addEventListener('click', (e) => {
        if (e.target === modal) this.closeDiagnosticsModal();
      });
    }

    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape' && modal?.classList.contains('open')) {
        this.closeDiagnosticsModal();
      }
    });
  }

  openDiagnosticsModal() {
    const modal = document.getElementById('diagnostics-modal');
    if (modal) {
      modal.classList.add('open');
      this.runDiagnostics();
    }
  }

  closeDiagnosticsModal() {
    const modal = document.getElementById('diagnostics-modal');
    if (modal) modal.classList.remove('open');
  }

  async runDiagnostics() {
    const list = document.getElementById('modal-diag-list');
    const summary = document.getElementById('modal-diag-summary');
    if (list) {
      list.innerHTML = `<div style="padding:24px; text-align:center; color:var(--text-muted); font-size:0.82rem;">Scanning network ports, binary runtimes, storage permissions & database integrity...</div>`;
    }

    try {
      const data = await this.api.getDiagnostics();
      if (summary) {
        summary.textContent = data.overall === 'healthy' 
          ? 'All subsystems healthy • Zero configuration conflicts'
          : `${data.issues_count} issue(s) detected • Guided solutions below`;
      }
      this.updateDiagnosticsUI(data);
      this.renderDiagnosticsModal(data);
    } catch (err) {
      if (list) {
        list.innerHTML = `<div style="padding:16px; color:#EF4444; font-size:0.8rem;">Failed to run diagnostics: ${err.message}</div>`;
      }
    }
  }

  async refreshDiagnostics() {
    try {
      const data = await this.api.getDiagnostics();
      this.updateDiagnosticsUI(data);
    } catch (_) {}
  }

  updateDiagnosticsUI(data) {
    const badge = document.getElementById('header-diag-badge');
    const dashCard = document.getElementById('dash-diagnostic-card');
    const dashBody = document.getElementById('dash-diag-body');
    const dashIcon = document.getElementById('dash-diag-icon');
    const dashHeadline = document.getElementById('dash-diag-headline');

    if (badge) {
      if (data.issues_count > 0) {
        badge.style.display = 'inline-flex';
        badge.textContent = data.issues_count;
        badge.className = data.overall === 'critical' ? 'diag-badge-pill critical' : 'diag-badge-pill';
      } else {
        badge.style.display = 'none';
      }
    }

    if (dashCard && dashBody) {
      if (data.issues_count > 0) {
        dashCard.style.display = 'block';
        dashCard.className = data.overall === 'critical' ? 'diagnostic-card' : 'diagnostic-card warning';
        if (dashIcon) dashIcon.textContent = data.overall === 'critical' ? '🚨' : '⚠️';
        if (dashHeadline) dashHeadline.textContent = data.overall === 'critical' ? 'Critical Setup Issue Detected' : 'Configuration Attention Required';

        const issueItems = data.items.filter(i => i.status !== 'ok');
        dashBody.innerHTML = issueItems.map(item => `
          <div class="diag-card-issue">
            <div style="font-size:0.82rem; font-weight:600; color:#FFF; display:flex; align-items:center; gap:6px;">
              <span>${item.status === 'error' ? '🔴' : '🟡'}</span>
              <span>${item.name}</span>
            </div>
            <div class="diag-section what">
              <span class="diag-section-label">What Happened</span>
              <span class="diag-section-text">${item.what}</span>
            </div>
            <div class="diag-section why">
              <span class="diag-section-label">Why It Might Have Happened</span>
              <span class="diag-section-text">${item.why}</span>
            </div>
            <div class="diag-section fix">
              <span class="diag-section-label">How You Can Fix That</span>
              <span class="diag-section-text">${item.fix}</span>
            </div>
            ${item.action_id ? `
              <div class="diag-action-bar">
                <button type="button" class="btn btn-secondary btn-sm btn-diag-fix" data-action="${item.action_id}">
                  Fix Issue Now →
                </button>
              </div>
            ` : ''}
          </div>
        `).join('');

        this.bindFixActionButtons(dashBody);
      } else {
        dashCard.style.display = 'none';
      }
    }
  }

  renderDiagnosticsModal(data) {
    const list = document.getElementById('modal-diag-list');
    if (!list) return;

    list.innerHTML = data.items.map(item => {
      const isOk = item.status === 'ok';
      return `
        <div class="diag-item-row ${item.status}">
          <div class="diag-item-header">
            <div style="display:flex; align-items:center; gap:8px;">
              <span>${isOk ? '🟢' : item.status === 'error' ? '🔴' : '🟡'}</span>
              <span class="diag-item-title">${item.name}</span>
            </div>
            <span class="mono-badge" style="font-size:0.7rem; color:${isOk ? 'var(--accent-emerald)' : item.status === 'error' ? '#EF4444' : 'var(--accent-amber)'};">
              ${isOk ? 'Healthy ✓' : item.status === 'error' ? 'Requires Attention' : 'Warning'}
            </span>
          </div>
          ${!isOk ? `
            <div style="display:flex; flex-direction:column; gap:8px; margin-top:8px;">
              <div class="diag-section what">
                <span class="diag-section-label">What Happened</span>
                <span class="diag-section-text">${item.what}</span>
              </div>
              <div class="diag-section why">
                <span class="diag-section-label">Why It Might Have Happened</span>
                <span class="diag-section-text">${item.why}</span>
              </div>
              <div class="diag-section fix">
                <span class="diag-section-label">How You Can Fix That</span>
                <span class="diag-section-text">${item.fix}</span>
              </div>
              ${item.action_id ? `
                <div class="diag-action-bar">
                  <button type="button" class="btn btn-primary btn-sm btn-diag-fix" data-action="${item.action_id}">
                    Apply Automated Fix →
                  </button>
                </div>
              ` : ''}
            </div>
          ` : `
            <div style="font-size:0.75rem; color:var(--text-muted); margin-top:4px;">
              ${item.what}
            </div>
          `}
        </div>
      `;
    }).join('');

    this.bindFixActionButtons(list);
  }

  bindFixActionButtons(container) {
    container.querySelectorAll('.btn-diag-fix').forEach(btn => {
      btn.addEventListener('click', async () => {
        const action = btn.getAttribute('data-action');
        btn.classList.add('is-loading');

        try {
          if (action === 'switch_port_8085') {
            await this.api.setPort(8085);
            this.showToast('Web port changed to 8085 successfully!', 'success');
          } else if (action === 'grant_dataroot_acl') {
            const dataDir = this.state?.data_dir || 'C:\\NextcloudAIO\\data';
            await this.api.setDataRoot(dataDir);
            this.showToast('Granted Full Control NTFS permissions to data root.', 'success');
          } else if (action === 'optimize_db') {
            await this.api.optimizeDatabase();
            this.showToast('WAL Checkpoint & VACUUM completed!', 'success');
          } else if (action === 'open_config_tunnel') {
            this.closeDiagnosticsModal();
            this.switchToView('view-config');
            document.getElementById('input-tunnel-token')?.focus();
            return;
          } else if (action.startsWith('unmount:')) {
            const targetPath = action.replace('unmount:', '');
            await this.api.unmountDrive(targetPath);
            this.showToast(`Unmounted missing drive '${targetPath}'.`, 'info');
          }
          await this.refresh();
          await this.runDiagnostics();
        } catch (e) {
          this.showToast(`Fix error: ${e.message}`, 'error');
        } finally {
          btn.classList.remove('is-loading');
        }
      });
    });
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

      if (sourceRect.width > 0 && targetRect.width > 0) {
        const deltaX = targetRect.left + (targetRect.width / 2) - (sourceRect.left + (sourceRect.width / 2));
        const deltaY = targetRect.top + (targetRect.height / 2) - (sourceRect.top + (sourceRect.height / 2));
        const scale = targetRect.width / sourceRect.width;

        splashLogoContainer.style.transform = `translate(${deltaX}px, ${deltaY}px) scale(${scale})`;
      }
      splash.classList.add('splash-leaving');

      setTimeout(() => {
        splash.style.display = 'none';
      }, 850);
    }, 850);
  }

  async refresh() {
    this.state = await this.api.getStatus();
    const adminData = await this.api.getAdminUser();

    // Synchronize live operational logs from supervisor backend
    try {
      const serverLogs = await this.api.getLogs();
      if (serverLogs && serverLogs.length > 0) {
        this.logViewer.syncLogs(serverLogs);
      }
    } catch (_) {}

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

    // Update Modern Hero Card Status
    const heroDot = document.getElementById('hero-dot');
    const heroTag = document.getElementById('hero-status-tag');
    const heroHeadline = document.getElementById('hero-headline');
    const heroDesc = document.getElementById('hero-desc');
    const heroBtnOpen = document.getElementById('hero-btn-open');
    const heroLocalUrl = document.getElementById('hero-local-url');
    const heroTunnelStatus = document.getElementById('hero-tunnel-status');

    // 2x2 Grid Component Status Elements
    const dotDashCore = document.getElementById('dot-dash-core');
    const lblDashCore = document.getElementById('lbl-dash-core');
    const dotDashCaddy = document.getElementById('dot-dash-caddy');
    const lblDashCaddy = document.getElementById('lbl-dash-caddy');
    const dotDashPhp = document.getElementById('dot-dash-php');
    const lblDashPhp = document.getElementById('lbl-dash-php');

    if (isRunning) {
      if (headerStatusDot) headerStatusDot.className = 'status-dot dot-running';
      if (headerStatusText) headerStatusText.textContent = 'Running';
      if (btnToggle) btnToggle.className = 'btn btn-destructive-outline';
      if (lblToggle) lblToggle.textContent = 'Stop Server';
      if (iconToggle) iconToggle.innerHTML = '<rect x="6" y="6" width="12" height="12"></rect>';
      if (btnOpenNextcloud) {
        btnOpenNextcloud.href = localUrl;
        btnOpenNextcloud.className = 'btn btn-primary';
        btnOpenNextcloud.removeAttribute('disabled');
      }

      if (heroDot) heroDot.className = 'status-dot dot-running';
      if (heroTag) heroTag.textContent = 'Nextcloud Stack Active';
      if (heroHeadline) heroHeadline.textContent = 'Nextcloud is Live & Healthy';
      if (heroDesc) heroDesc.textContent = 'Your personal cloud is running on this Windows PC. File sync, desktop collaboration, and media streaming are operational.';
      if (heroBtnOpen) {
        heroBtnOpen.href = localUrl;
        heroBtnOpen.className = 'btn btn-primary';
        heroBtnOpen.removeAttribute('disabled');
      }

      if (dotDashCore) dotDashCore.className = 'status-dot dot-running';
      if (lblDashCore) lblDashCore.textContent = 'Running';
      if (dotDashCaddy) dotDashCaddy.className = 'status-dot dot-running';
      if (lblDashCaddy) lblDashCaddy.textContent = 'Active';
      if (dotDashPhp) dotDashPhp.className = 'status-dot dot-running';
      if (lblDashPhp) lblDashPhp.textContent = '127.0.0.1:9000';
    } else {
      if (headerStatusDot) headerStatusDot.className = 'status-dot dot-stopped';
      if (headerStatusText) headerStatusText.textContent = 'Stopped';
      if (btnToggle) btnToggle.className = 'btn btn-success';
      if (lblToggle) lblToggle.textContent = 'Start Server';
      if (iconToggle) iconToggle.innerHTML = '<polygon points="5 3 19 12 5 21 5 3"></polygon>';
      if (btnOpenNextcloud) {
        btnOpenNextcloud.href = '#';
        btnOpenNextcloud.className = 'btn btn-primary disabled';
        btnOpenNextcloud.setAttribute('disabled', 'true');
      }

      if (heroDot) heroDot.className = 'status-dot dot-stopped';
      if (heroTag) heroTag.textContent = 'Nextcloud Services Stopped';
      if (heroHeadline) heroHeadline.textContent = 'Nextcloud is Paused';
      if (heroDesc) heroDesc.textContent = 'Web server and background sync are currently paused. Click "Start Server" above to resume.';
      if (heroBtnOpen) {
        heroBtnOpen.href = '#';
        heroBtnOpen.className = 'btn btn-primary disabled';
        heroBtnOpen.setAttribute('disabled', 'true');
      }

      if (dotDashCore) dotDashCore.className = 'status-dot dot-stopped';
      if (lblDashCore) lblDashCore.textContent = 'Stopped';
      if (dotDashCaddy) dotDashCaddy.className = 'status-dot dot-stopped';
      if (lblDashCaddy) lblDashCaddy.textContent = 'Stopped';
      if (dotDashPhp) dotDashPhp.className = 'status-dot dot-stopped';
      if (lblDashPhp) lblDashPhp.textContent = 'Offline';
    }

    if (heroLocalUrl) {
      heroLocalUrl.href = localUrl;
      heroLocalUrl.textContent = localUrl;
      const copyLocalBtn = document.getElementById('hero-btn-copy-local');
      if (copyLocalBtn) copyLocalBtn.setAttribute('data-copy', localUrl);
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
    if (adminUserElem && adminData?.username) {
      adminUserElem.textContent = adminData.username;
    }

    // Update Port Field
    const inputPort = document.getElementById('input-server-port');
    if (inputPort && this.state.server_port) {
      inputPort.value = this.state.server_port;
    }

    // Update Tunnel displays
    const isTunnelActive = Boolean(this.state.tunnel_status === 'Active' || this.state.tunnel_status === 'Connected');
    const tunnelUrl = this.state.tunnel_url || 'https://cloud.yourdomain.com';

    const dotDashTunnel = document.getElementById('dot-dash-tunnel');
    const lblDashTunnelStatus = document.getElementById('lbl-dash-tunnel-status');
    const lblDashTunnelDesc = document.getElementById('lbl-dash-tunnel-desc');
    const rowDashTunnelEndpoint = document.getElementById('row-dash-tunnel-endpoint');
    const btnJumpTunnel = document.getElementById('btn-jump-tunnel');
    const btnDisconnectTunnel = document.getElementById('btn-disconnect-tunnel');
    const cfgTunnelUrl = document.getElementById('lbl-tunnel-active-url');
    const dotCfgTunnel = document.getElementById('dot-cfg-tunnel');

    if (isTunnelActive) {
      if (heroTunnelStatus) {
        heroTunnelStatus.textContent = 'Connected • Encrypted';
        heroTunnelStatus.className = 'hero-meta-val text-emerald';
      }
      if (dotDashTunnel) dotDashTunnel.className = 'status-dot dot-running';
      if (lblDashTunnelStatus) lblDashTunnelStatus.textContent = 'Connected';
      if (lblDashTunnelDesc) lblDashTunnelDesc.textContent = 'HTTPS encrypted tunnel active via Cloudflare edge.';
      if (rowDashTunnelEndpoint) rowDashTunnelEndpoint.style.display = 'flex';
      if (btnJumpTunnel) btnJumpTunnel.style.display = 'none';
      if (btnDisconnectTunnel) btnDisconnectTunnel.style.display = 'inline-block';

      const tunnelUrlElem = document.getElementById('lbl-tunnel-url');
      if (tunnelUrlElem) {
        tunnelUrlElem.href = tunnelUrl;
        tunnelUrlElem.textContent = tunnelUrl;
        const copyTunnelBtn = document.getElementById('btn-copy-tunnel');
        if (copyTunnelBtn) copyTunnelBtn.setAttribute('data-copy', tunnelUrl);
      }

      if (cfgTunnelUrl) {
        cfgTunnelUrl.href = tunnelUrl;
        cfgTunnelUrl.textContent = tunnelUrl;
      }
      if (dotCfgTunnel) dotCfgTunnel.className = 'status-dot dot-running';
    } else {
      if (heroTunnelStatus) {
        heroTunnelStatus.textContent = 'Local Only';
        heroTunnelStatus.className = 'hero-meta-val';
      }
      if (dotDashTunnel) dotDashTunnel.className = 'status-dot dot-stopped';
      if (lblDashTunnelStatus) lblDashTunnelStatus.textContent = 'Not Configured';
      if (lblDashTunnelDesc) lblDashTunnelDesc.textContent = 'Remote access is disabled. Nextcloud is accessible only on your local network.';
      if (rowDashTunnelEndpoint) rowDashTunnelEndpoint.style.display = 'none';
      if (btnJumpTunnel) btnJumpTunnel.style.display = 'inline-flex';
      if (btnDisconnectTunnel) btnDisconnectTunnel.style.display = 'none';

      if (cfgTunnelUrl) {
        cfgTunnelUrl.removeAttribute('href');
        cfgTunnelUrl.textContent = 'Not Connected';
      }
      if (dotCfgTunnel) dotCfgTunnel.className = 'status-dot dot-stopped';
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

    // Render Auto-Detected System Drives (1-Click Mount Chips)
    try {
      const detectedDrives = await this.api.getAvailableSystemDrives();
      const detectedContainer = document.getElementById('detected-drives-container');
      const detectedCountLbl = document.getElementById('lbl-detected-drives-count');
      
      if (detectedContainer) {
        if (detectedDrives.length === 0) {
          detectedContainer.innerHTML = `<span style="font-size:0.75rem; color:var(--text-muted); font-family:var(--font-mono);">No unmounted external volumes detected.</span>`;
          if (detectedCountLbl) detectedCountLbl.textContent = '0 available';
        } else {
          if (detectedCountLbl) detectedCountLbl.textContent = `${detectedDrives.length} detected`;
          detectedContainer.innerHTML = detectedDrives.map(d => `
            <div class="drive-chip" data-letter="${d.letter}" data-path="${d.path}" data-mounted="${d.is_mounted}">
              <span class="chip-letter">${d.letter}\\</span>
              <span class="chip-free">${d.free_gb} GB free</span>
              <span class="chip-action ${d.is_mounted ? 'mounted' : 'unmounted'}">${d.is_mounted ? 'Mounted ✓' : '+ 1-Click Mount'}</span>
            </div>
          `).join('');

          // Bind 1-click mount chip handlers with instant mounting
          detectedContainer.querySelectorAll('.drive-chip').forEach(chip => {
            chip.addEventListener('click', async () => {
              const pathVal = chip.getAttribute('data-path');
              const letterVal = chip.getAttribute('data-letter');
              const isMounted = chip.getAttribute('data-mounted') === 'true';

              if (isMounted) {
                this.showToast(`Drive ${letterVal}\\ is already mounted. Browse it in Connected Drives below.`, 'info');
                return;
              }

              chip.style.opacity = '0.6';
              this.showToast(`Mounting drive ${letterVal}\\ with NTFS permissions...`, 'info');
              try {
                await this.api.mountDrive(pathVal, `Drive ${letterVal}`);
                this.showToast(`Drive ${letterVal}\\ mounted successfully!`, 'success');
                await this.refresh();
              } catch (err) {
                this.showToast(`Failed to mount ${letterVal}\\: ${err.message}`, 'error');
              }
            });
          });
        }
      }
    } catch (_) {}

    // Render Database Status & Health
    try {
      const dbStatus = await this.api.getDatabaseStatus();
      const lblDbEngine = document.getElementById('lbl-db-engine');
      if (lblDbEngine && dbStatus.engine) lblDbEngine.textContent = dbStatus.engine;

      const lblDbSize = document.getElementById('lbl-db-size');
      if (lblDbSize) {
        const mb = (dbStatus.file_size_bytes / (1024 * 1024)).toFixed(2);
        lblDbSize.textContent = `${mb} MB`;
      }

      const lblDbWalSize = document.getElementById('lbl-db-wal-size');
      if (lblDbWalSize) {
        const walMb = (dbStatus.wal_size_bytes / (1024 * 1024)).toFixed(2);
        lblDbWalSize.textContent = `${walMb} MB`;
      }

      const lblDbIntegrity = document.getElementById('lbl-db-integrity');
      if (lblDbIntegrity && dbStatus.integrity) {
        lblDbIntegrity.textContent = dbStatus.integrity;
        lblDbIntegrity.style.color = dbStatus.integrity === 'ok' ? 'var(--accent-emerald)' : 'var(--accent-amber)';
      }

      const dotDbStatus = document.getElementById('dot-db-status');
      if (dotDbStatus) {
        dotDbStatus.className = dbStatus.integrity === 'ok' ? 'status-dot dot-running' : 'status-dot dot-warning';
      }

      const lblDbStatus = document.getElementById('lbl-db-status');
      if (lblDbStatus) {
        const mode = (dbStatus.journal_mode || 'wal').toUpperCase();
        lblDbStatus.textContent = `${mode} Active • ${dbStatus.integrity === 'ok' ? 'ACID Healthy' : 'Check Integrity'}`;
      }
    } catch (_) {}

    // Render Windows Auto-Start on Boot Status
    try {
      const autoData = await this.api.getAutostart();
      this.updateAutostartUI(autoData.enabled);
    } catch (_) {}

    // Render Drive Cards with single subtle metadata label, Open in Explorer, and unmount
    const driveContainer = document.getElementById('drive-list-container');
    if (driveContainer && this.state.drives) {
      if (this.state.drives.length === 0) {
        driveContainer.innerHTML = `<div style="padding:14px; text-align:center; color:var(--text-muted); font-size:0.78rem; border:1px dashed var(--border-card); border-radius:6px;">No external storage drives mounted yet. Select a detected drive above or enter a folder path to mount.</div>`;
      } else {
        driveContainer.innerHTML = this.state.drives.map(d => {
          const total = Math.max(1, d.total_gb || 0);
          const used = Math.max(0, d.used_gb || 0);
          const free = d.free_gb != null ? d.free_gb : Math.max(0, total - used);
          const pct = Math.min(100, Math.max(0, Math.round((used / total) * 100)));
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
              <div style="display:flex; align-items:center; gap:8px;">
                <span class="drive-meta-label">NTFS • Mounted</span>
                <button type="button" class="btn-open-drive-folder btn btn-secondary btn-sm" data-path="${d.path}" title="Open folder in Windows File Explorer" style="padding:3px 8px; font-size:0.74rem;">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
                  Explorer
                </button>
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

        // Bind Open in Explorer handlers
        driveContainer.querySelectorAll('.btn-open-drive-folder').forEach(btn => {
          btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            const drivePath = btn.getAttribute('data-path');
            await this.api.openFolder(drivePath);
            this.showToast(`Opened '${drivePath}' in Windows Explorer`, 'success');
          });
        });

        // Bind unmount click handlers
        driveContainer.querySelectorAll('.btn-unmount-drive').forEach(btn => {
          btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            const drivePath = btn.getAttribute('data-path');
            if (confirm(`Unmount external drive '${drivePath}' from Nextcloud?`)) {
              this.logViewer.append('CORE', `Unmounting external drive: ${drivePath}...`);
              await this.api.unmountDrive(drivePath);
              this.logViewer.append('CORE', `Drive '${drivePath}' unmounted.`);
              this.showToast(`Drive '${drivePath}' unmounted.`, 'info');
              await this.refresh();
            }
          });
        });
      }
    }

    // Refresh live health and self-healing diagnostics
    await this.refreshDiagnostics();
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
        this.showToast(`Removed trusted domain: ${dom}`, 'info');
        this.refreshDomains();
      });
    });
  }
}

// Bootstrap application on DOM ready
document.addEventListener('DOMContentLoaded', () => {
  window.nextcloudApp = new AppController();
});
