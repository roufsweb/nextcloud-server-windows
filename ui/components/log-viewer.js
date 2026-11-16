/**
 * In-App High Performance Log Viewer
 * Implements a ring-buffered virtual console with tag filtering and auto-scroll
 */

export class LogViewer {
  constructor(containerId) {
    this.container = document.getElementById(containerId);
    this.buffer = [];
    this.maxLines = 500;
    this.filter = 'all';
    this.autoScroll = true;
    this.init();
  }

  init() {
    this.container.innerHTML = `
      <div class="log-viewer-container">
        <div class="log-toolbar">
          <div class="log-filters">
            <button class="log-filter-btn active" data-filter="all">All Logs</button>
            <button class="log-filter-btn" data-filter="CORE">Core</button>
            <button class="log-filter-btn" data-filter="CADDY">Caddy</button>
            <button class="log-filter-btn" data-filter="PHP">PHP</button>
            <button class="log-filter-btn" data-filter="TUNNEL">Cloudflared</button>
          </div>
          <div style="display:flex; gap:8px;">
            <button class="btn btn-secondary btn-sm" id="btn-copy-logs">Copy</button>
            <button class="btn btn-secondary btn-sm" id="btn-clear-logs">Clear</button>
          </div>
        </div>
        <div class="log-console" id="log-console-body"></div>
      </div>
    `;

    this.consoleBody = document.getElementById('log-console-body');

    // Filter clicks
    this.container.querySelectorAll('.log-filter-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        this.container.querySelectorAll('.log-filter-btn').forEach(b => b.classList.remove('active'));
        e.target.classList.add('active');
        this.filter = e.target.getAttribute('data-filter');
        this.render();
      });
    });

    document.getElementById('btn-clear-logs').addEventListener('click', () => {
      this.buffer = [];
      this.render();
    });

    document.getElementById('btn-copy-logs').addEventListener('click', () => {
      const rawText = this.buffer.map(l => `[${l.time}] [${l.tag}] ${l.message}`).join('\n');
      navigator.clipboard.writeText(rawText);
      alert('Logs copied to clipboard.');
    });
  }

  append(tag, message) {
    const time = new Date().toLocaleTimeString();
    this.buffer.push({ time, tag, message });
    if (this.buffer.length > this.maxLines) {
      this.buffer.shift();
    }
    this.render();
  }

  render() {
    const filtered = this.filter === 'all' 
      ? this.buffer 
      : this.buffer.filter(item => item.tag.toUpperCase() === this.filter);

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
    return str.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  }
}
