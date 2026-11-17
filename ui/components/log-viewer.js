/**
 * In-App High Performance Log Viewer
 * Implements segmented controls, instant search filtering, and column-aligned monospace streaming
 */

export class LogViewer {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
    this.buffer = [];
    this.maxLines = 500;
    this.filter = 'all';
    this.searchQuery = '';
    this.autoScroll = true;
    this.init();
  }

  init() {
    if (!this.container) return;

    this.container.innerHTML = `
      <div class="log-viewer-container">
        <div class="log-toolbar">
          <div class="segmented-control">
            <button class="log-filter-btn active" data-filter="all">All Logs</button>
            <button class="log-filter-btn" data-filter="CORE">Core</button>
            <button class="log-filter-btn" data-filter="CADDY">Caddy</button>
            <button class="log-filter-btn" data-filter="PHP">PHP</button>
            <button class="log-filter-btn" data-filter="TUNNEL">Cloudflared</button>
          </div>
          <div style="display:flex; align-items:center; gap:8px;">
            <input type="text" class="log-search-input" id="input-log-search" placeholder="Filter logs..." autocomplete="off" spellcheck="false">
            <button class="btn btn-secondary btn-sm" id="btn-copy-logs" title="Copy visible logs">Copy Logs</button>
            <button class="btn btn-secondary btn-sm" id="btn-clear-logs" title="Clear buffer">Clear</button>
          </div>
        </div>
        <div class="log-console" id="log-console-body"></div>
      </div>
    `;

    this.consoleBody = document.getElementById('log-console-body');
    const searchInput = document.getElementById('input-log-search');

    // Filter button clicks
    this.container.querySelectorAll('.log-filter-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        this.container.querySelectorAll('.log-filter-btn').forEach(b => b.classList.remove('active'));
        btn.classList.add('active');
        this.filter = btn.getAttribute('data-filter');
        this.render();
      });
    });

    // Instant search input listener
    if (searchInput) {
      searchInput.addEventListener('input', (e) => {
        this.searchQuery = e.target.value.toLowerCase().trim();
        this.render();
      });
    }

    // Clear logs
    const btnClear = document.getElementById('btn-clear-logs');
    if (btnClear) {
      btnClear.addEventListener('click', () => {
        this.buffer = [];
        this.render();
      });
    }

    // Copy logs
    const btnCopy = document.getElementById('btn-copy-logs');
    if (btnCopy) {
      btnCopy.addEventListener('click', () => {
        const lines = this.getFilteredLines();
        const rawText = lines.map(l => `[${l.time}] [${l.tag.padEnd(6)}] ${l.message}`).join('\n');
        navigator.clipboard.writeText(rawText);
        const originalText = btnCopy.textContent;
        btnCopy.textContent = 'Copied!';
        setTimeout(() => { btnCopy.textContent = originalText; }, 1500);
      });
    }
  }

  append(tag, message) {
    const time = new Date().toTimeString().split(' ')[0];
    this.buffer.push({ time, tag: tag.toUpperCase(), message });
    if (this.buffer.length > this.maxLines) {
      this.buffer.shift();
    }
    this.render();
  }

  getFilteredLines() {
    return this.buffer.filter(item => {
      const matchesFilter = (this.filter === 'all' || item.tag === this.filter);
      const matchesSearch = !this.searchQuery || 
        item.message.toLowerCase().includes(this.searchQuery) ||
        item.tag.toLowerCase().includes(this.searchQuery);
      return matchesFilter && matchesSearch;
    });
  }

  render() {
    if (!this.consoleBody) return;

    const filtered = this.getFilteredLines();

    if (filtered.length === 0) {
      this.consoleBody.innerHTML = `<div style="color:var(--text-muted); font-size:0.75rem; padding:8px 0;">No logs match current filter.</div>`;
      return;
    }

    this.consoleBody.innerHTML = filtered.map(item => {
      const tagClass = `tag-${item.tag.toLowerCase()}`;
      return `
        <div class="log-line">
          <span class="log-time">${item.time}</span>
          <span class="log-tag ${tagClass}">[${item.tag}]</span>
          <span class="log-msg">${this.escapeHtml(item.message)}</span>
        </div>
      `;
    }).join('');

    if (this.autoScroll) {
      this.consoleBody.scrollTop = this.consoleBody.scrollHeight;
    }
  }

  escapeHtml(str) {
    return String(str)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }
}
