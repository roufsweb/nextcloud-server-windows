/**
 * In-App High Performance Log Viewer
 * Implements segmented controls, instant search filtering, pause streaming, and virtualized / fast DOM streaming
 */

export class LogViewer {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
    this.buffer = [];
    this.maxLines = 500;
    this.filter = 'all';
    this.searchQuery = '';
    this.autoScroll = true;
    this.isPaused = false;
    this.seenKeys = new Set();
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
            <button class="btn btn-secondary btn-sm" id="btn-pause-logs" title="Pause or resume live log stream">Pause</button>
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
      btn.addEventListener('click', () => {
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

    // Pause Stream Button
    const btnPause = document.getElementById('btn-pause-logs');
    if (btnPause) {
      btnPause.addEventListener('click', () => {
        this.isPaused = !this.isPaused;
        btnPause.textContent = this.isPaused ? 'Resume' : 'Pause';
        if (this.isPaused) {
          btnPause.classList.add('btn-warning');
        } else {
          btnPause.classList.remove('btn-warning');
          this.render();
        }
      });
    }

    // Scroll listener for sticky bottom auto-scroll
    if (this.consoleBody) {
      this.consoleBody.addEventListener('scroll', () => {
        const threshold = 40;
        const distFromBottom = this.consoleBody.scrollHeight - this.consoleBody.scrollTop - this.consoleBody.clientHeight;
        this.autoScroll = distFromBottom <= threshold;
      });
    }

    // Clear logs
    const btnClear = document.getElementById('btn-clear-logs');
    if (btnClear) {
      btnClear.addEventListener('click', () => {
        this.buffer = [];
        this.seenKeys.clear();
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

  append(tag, message, timeOverride = null) {
    const time = timeOverride || new Date().toTimeString().split(' ')[0];
    const key = `${time}|${tag}|${message}`;
    if (this.seenKeys.has(key)) return;
    this.seenKeys.add(key);

    const item = { time, tag: tag.toUpperCase(), message };
    this.buffer.push(item);
    if (this.buffer.length > this.maxLines) {
      const removed = this.buffer.shift();
      if (removed) {
        this.seenKeys.delete(`${removed.time}|${removed.tag}|${removed.message}`);
      }
    }

    if (this.isPaused) return;

    // Fast O(1) DOM append when unfiltered
    if (this.filter === 'all' && !this.searchQuery && this.consoleBody) {
      const lineEl = document.createElement('div');
      lineEl.className = 'log-line';
      lineEl.innerHTML = `
        <span class="log-time">${item.time}</span>
        <span class="log-tag tag-${item.tag.toLowerCase()}">[${item.tag}]</span>
        <span class="log-msg">${this.escapeHtml(item.message)}</span>
      `;
      this.consoleBody.appendChild(lineEl);
      while (this.consoleBody.children.length > this.maxLines) {
        this.consoleBody.removeChild(this.consoleBody.firstChild);
      }
      if (this.autoScroll) {
        this.consoleBody.scrollTop = this.consoleBody.scrollHeight;
      }
    } else {
      this.render();
    }
  }

  syncLogs(newLogs = []) {
    if (!Array.isArray(newLogs) || newLogs.length === 0) return;
    let added = false;
    for (const log of newLogs) {
      const time = log.time || new Date().toTimeString().split(' ')[0];
      const tag = (log.tag || 'CORE').toUpperCase();
      const message = log.message || '';
      const key = `${time}|${tag}|${message}`;
      if (!this.seenKeys.has(key)) {
        this.seenKeys.add(key);
        this.buffer.push({ time, tag, message });
        added = true;
      }
    }
    while (this.buffer.length > this.maxLines) {
      const removed = this.buffer.shift();
      if (removed) {
        this.seenKeys.delete(`${removed.time}|${removed.tag}|${removed.message}`);
      }
    }
    if (added && !this.isPaused) {
      this.render();
    }
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
