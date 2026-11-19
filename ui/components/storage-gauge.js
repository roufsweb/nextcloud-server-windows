/**
 * Storage Gauge and Allocation Component
 * Production-grade developer utility style (Linear/Raycast inspired)
 */

export class StorageGauge {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
    this.lastRenderKey = null;
  }

  render(data) {
    if (!this.container) return;

    const totalGB = Math.max(1, data?.total_disk_gb || 2000);
    const usedGB = Math.max(0, data?.used_disk_gb || 0);
    const freeGB = Math.max(0, totalGB - usedGB);
    const pctUsed = Math.min(100, Math.max(0, Math.round((usedGB / totalGB) * 100)));

    const renderKey = `${usedGB}-${totalGB}-${pctUsed}`;

    // High performance in-place update if DOM structure is already mounted
    const existingHeader = this.container.querySelector('.storage-metrics-header');
    const existingFill = this.container.querySelector('.storage-bar-fill');
    const existingNextcloudData = this.container.querySelector('.legend-used-text');
    const existingFreeSpace = this.container.querySelector('.legend-free-text');

    if (existingHeader && existingFill && existingNextcloudData && existingFreeSpace) {
      if (this.lastRenderKey === renderKey) return;
      this.lastRenderKey = renderKey;
      existingHeader.textContent = `${usedGB} GB / ${totalGB} GB (${pctUsed}%)`;
      existingFill.style.width = `${pctUsed}%`;
      existingNextcloudData.textContent = `Nextcloud Data (${usedGB} GB)`;
      existingFreeSpace.textContent = `Free Space (${freeGB} GB)`;
      return;
    }

    this.lastRenderKey = renderKey;
    this.container.innerHTML = `
      <div class="card">
        <div class="card-header">
          <div class="card-title">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect>
              <rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect>
              <line x1="6" y1="6" x2="6.01" y2="6"></line>
              <line x1="6" y1="18" x2="6.01" y2="18"></line>
            </svg>
            Storage & Disk Allocation
          </div>
          <span class="storage-metrics-header">${usedGB} GB / ${totalGB} GB (${pctUsed}%)</span>
        </div>

        <div class="storage-overview">
          <div class="storage-bar-track">
            <div class="storage-bar-fill" style="width: ${pctUsed}%;"></div>
          </div>
          <div class="storage-legend">
            <div class="legend-item">
              <span class="legend-dot" style="background: var(--accent-sky);"></span>
              <span class="legend-used-text">Nextcloud Data (${usedGB} GB)</span>
            </div>
            <div class="legend-item">
              <span class="legend-dot" style="background: #27272A;"></span>
              <span class="legend-free-text">Free Space (${freeGB} GB)</span>
            </div>
          </div>
        </div>
      </div>
    `;
  }
}
