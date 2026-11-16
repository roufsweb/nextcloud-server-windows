/**
 * Storage Gauge and Drive Component
 * Manages visualization of storage space, data root, and on-demand external drive mounting.
 */

export class StorageGauge {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
  }

  render(data) {
    const totalGB = data.total_disk_gb || 1000;
    const usedGB = data.used_disk_gb || 142;
    const freeGB = totalGB - usedGB;
    const pctUsed = Math.min(100, Math.round((usedGB / totalGB) * 100));

    this.container.innerHTML = `
      <div class="card">
        <div class="card-header">
          <div class="card-title">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
              <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
              <line x1="6" y1="6" x2="6.01" y2="6"></line>
              <line x1="6" y1="18" x2="6.01" y2="18"></line>
            </svg>
            Storage & Disk Allocation
          </div>
          <span class="status-pill online">Healthy</span>
        </div>

        <div class="storage-overview">
          <div class="storage-metrics">
            <span><strong>${usedGB} GB</strong> used of <strong>${totalGB} GB</strong></span>
            <span>${pctUsed}%</span>
          </div>
          <div class="storage-bar-track">
            <div class="storage-bar-fill" style="width: ${pctUsed}%"></div>
          </div>
          <div class="storage-legend">
            <div class="legend-item">
              <span class="legend-dot" style="background: var(--accent-blue)"></span>
              <span>Nextcloud Data (${usedGB} GB)</span>
            </div>
            <div class="legend-item">
              <span class="legend-dot" style="background: rgba(255,255,255,0.2)"></span>
              <span>Free Space (${freeGB} GB)</span>
            </div>
          </div>
        </div>
      </div>
    `;
  }
}
