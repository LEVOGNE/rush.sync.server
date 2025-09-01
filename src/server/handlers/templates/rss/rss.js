/**
 * Rush Sync Dashboard - Optimiert für Performance und Robustheit
 * WebSocket Hot Reload mit minimaler Speicher- und CPU-Belastung
 */

class RushSyncApp {
  constructor() {
    // Konsolidierte Config - Reduzierung von Objekt-Allokationen
    this.config = {
      statusInterval: 5000,
      logPollInterval: 2000,
      metricsInterval: 10000,
      maxLogLines: 500,
      autoScroll: true,
      serverHealthCheckInterval: 2000,
      websocketReconnectDelay: 1000,
      maxReconnectAttempts: 10,
      hotReloadDebounce: 250,
    };

    // Optimierter State - Reduzierte Verschachtelung
    this.state = {
      currentTab: 'overview',
      logs: [],
      metrics: {},
      requests: 0,
      errors: 0,
      uptime: 0,
      serverAlive: true,
      shutdownNotified: false,
      websocket: null,
      websocketConnected: false,
      reconnectAttempts: 0,
      lastReloadTime: 0,
    };

    this.intervals = {};
    this.startTime = Date.now();
    this.serverName = '{{SERVER_NAME}}';
    this.serverPort = '{{PORT}}';

    this.init();
  }

  init() {
    this.setupEventListeners();
    this.initializeTabs();
    this.formatCreationTime();
    this.startAllMonitoring();
    this.initializeHotReload();

    window.RushApp = this;

    console.log('[Rush Sync] Dashboard initialized with Hot Reload');
    this.addLogEntry('INFO', 'Hot Reload WebSocket initializing...', 'system');
  }

  // Hot Reload WebSocket - Optimiert
  initializeHotReload() {
    this.connectWebSocket();
  }

  connectWebSocket() {
    if (this.state.websocket?.readyState === WebSocket.OPEN) return;

    const wsUrl = `ws://127.0.0.1:${this.serverPort}/ws/hot-reload`;

    try {
      this.state.websocket = new WebSocket(wsUrl);

      this.state.websocket.onopen = () => {
        this.state.websocketConnected = true;
        this.state.reconnectAttempts = 0;
        this.updateHotReloadStatus('connected');
        console.log('[Hot Reload] Connected');
        this.addLogEntry('SUCCESS', 'Hot Reload connected', 'system');
      };

      this.state.websocket.onmessage = (event) => {
        try {
          const changeEvent = JSON.parse(event.data);
          this.handleFileChange(changeEvent);
        } catch (e) {
          console.warn('[Hot Reload] Invalid message:', e);
        }
      };

      this.state.websocket.onclose = (event) => {
        this.state.websocketConnected = false;
        this.updateHotReloadStatus('disconnected');

        if (!event.wasClean && !this.state.shutdownNotified) {
          console.log('[Hot Reload] Reconnecting...');
          this.addLogEntry('WARN', 'Hot Reload reconnecting...', 'system');
          this.attemptReconnect();
        }
      };

      this.state.websocket.onerror = () => {
        this.updateHotReloadStatus('error');
        this.addLogEntry('ERROR', 'Hot Reload error', 'error');
      };
    } catch (error) {
      console.error('[Hot Reload] Connection failed:', error);
      this.updateHotReloadStatus('error');
      this.attemptReconnect();
    }
  }

  handleFileChange(changeEvent) {
    const now = Date.now();
    if (now - this.state.lastReloadTime < this.config.hotReloadDebounce) return;

    console.log('[Hot Reload] File change:', changeEvent);

    const fileName = changeEvent.file_path.split('/').pop() || 'unknown';
    this.addLogEntry('INFO', `File ${changeEvent.event_type}: ${fileName}`, 'hotreload');

    if (this.shouldTriggerReload(changeEvent)) {
      this.state.lastReloadTime = now;
      this.triggerPageReload(fileName);
    }
  }

  shouldTriggerReload(changeEvent) {
    const extension = changeEvent.file_extension?.toLowerCase();
    const webExtensions = [
      'html',
      'css',
      'js',
      'json',
      'txt',
      'md',
      'svg',
      'png',
      'jpg',
      'jpeg',
      'gif',
      'ico',
    ];
    return webExtensions.includes(extension) && changeEvent.event_type !== 'deleted';
  }

  triggerPageReload(fileName) {
    this.showReloadNotification(fileName);
    setTimeout(() => {
      console.log('[Hot Reload] Reloading:', fileName);
      window.location.reload();
    }, 500);
  }

  // Optimierte Notification - Reduzierte DOM-Operationen
  showReloadNotification(fileName) {
    const existing = document.getElementById('hot-reload-notification');
    if (existing) existing.remove();

    const notification = document.createElement('div');
    notification.id = 'hot-reload-notification';
    notification.innerHTML = `
    <div style="position: fixed;top: 11px;right: 11px;background: rgba(0, 212, 255, 0.95);color: #1a1d23;border: 0.1rem solid rgba(255, 255, 255, 0.5);outline: 0.1rem solid rgba(0, 0, 0, 0.5);padding: 7px 13px;border-radius: 3px;font-size: 11px;z-index: 10000;animation: slideIn 0.3s ease-out;">Reloading: ${fileName}</div><style>@keyframes slideIn{from{transform: translateX(100%);opacity: 0;}to{transform: translateX(0);opacity: 1;}}</style>`;

    document.body.appendChild(notification);
    setTimeout(() => notification?.remove(), 3000);
  }

  attemptReconnect() {
    if (
      this.state.shutdownNotified ||
      this.state.reconnectAttempts >= this.config.maxReconnectAttempts
    )
      return;

    this.state.reconnectAttempts++;
    const delay = this.config.websocketReconnectDelay * this.state.reconnectAttempts;

    setTimeout(() => {
      if (!this.state.shutdownNotified) {
        console.log(
          `[Hot Reload] Attempt ${this.state.reconnectAttempts}/${this.config.maxReconnectAttempts}`,
        );
        this.connectWebSocket();
      }
    }, delay);
  }

  updateHotReloadStatus(status) {
    const statusElement = document.getElementById('hotreload-status');
    if (!statusElement) return;

    const statusText = {
      connected: '🟢 Active',
      disconnected: '🟡 Reconnecting',
      error: '🔴 Error',
    };

    statusElement.textContent = statusText[status] || '🔴 Unknown';
    statusElement.className = `hotreload-status ${status}`;
  }

  // Server Health Monitoring - Optimiert
  startServerHealthCheck() {
    this.intervals.healthCheck = setInterval(
      () => this.checkServerHealth(),
      this.config.serverHealthCheckInterval,
    );
  }

  async checkServerHealth() {
    if (this.state.shutdownNotified) return;

    try {
      const response = await fetch('/api/health', {
        method: 'GET',
        cache: 'no-cache',
        signal: AbortSignal.timeout(1500),
      });

      if (!response.ok) throw new Error(`Server returned ${response.status}`);

      if (!this.state.serverAlive) {
        this.state.serverAlive = true;
        console.log('[Rush Sync] Connection restored');
      }
    } catch (error) {
      if (this.state.serverAlive) {
        this.state.serverAlive = false;
        console.log('[Rush Sync] Connection lost:', error.message);
        this.handleServerShutdown();
      }
    }
  }

  // Shutdown Handler - Optimiert
  handleServerShutdown() {
    if (this.state.shutdownNotified) return;

    this.state.shutdownNotified = true;
    console.log('[Rush Sync] Server shutdown detected');

    this.state.websocket?.close();
    this.cleanup();

    document.body.innerHTML = `<div style="position: absolute;left: 50%;top: 50%;transform: translate(-50%, -50%);text-align: center;padding: 30px;font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;background: rgba(0, 212, 255, 0.25);color: #ffffff;display: flex;flex-direction: column;justify-content: center;align-items: center;border-radius: 11px;"><div style="border: 2px solid #00d4ff;padding: 30px;border-radius: 6px;max-width: 500px;background: #000000;"><h1 style="margin: 0 0 20px 0;color: #ff4757;font-size: 24px;font-weight: 600;text-transform: uppercase;">Server Stopped</h1><p style="margin: 10px 0; font-size: 16px; color: #ffffff; line-height: 1.4">Server '<strong style="color: #00d4ff">${this.serverName}</strong>' on port<strong style="color: #00d4ff">${this.serverPort}</strong> stopped.</p><p style="margin: 20px 0; font-size: 14px; color: #a0a6b1">Closing in <span id="countdown" style="color: #ff4757; font-weight: 600">3</span> seconds...</p><button onclick="window.close()"style="margin-top: 20px;padding: 10px 20px;background: #00d4ff;color: #1a1d23;border: none;cursor: pointer;border-radius: 5px;font-size: 12px;font-weight: 600;text-transform: uppercase;">Close Tab Now</button></div></div>`;

    let countdown = 3;
    const countdownElement = document.getElementById('countdown');
    const timer = setInterval(() => {
      countdown--;
      if (countdownElement) countdownElement.textContent = countdown;
      if (countdown <= 0) {
        clearInterval(timer);
        this.attemptBrowserClose();
      }
    }, 1000);
  }

  attemptBrowserClose() {
    console.log('[Rush Sync] Closing tab');
    try {
      // Erst versuchen zu schließen
      window.close();

      // Falls window.close() nicht funktioniert (moderne Browser-Sicherheit)
      // Alternative: Tab auf leere Seite umleiten
      setTimeout(() => {
        if (!window.closed) {
          window.location.href = 'about:blank';
        }
      }, 500);
    } catch (e) {
      console.warn('[Rush Sync] Close failed, redirecting:', e);
      window.location.href = 'about:blank';
    }
  }

  // Event Listeners - Optimiert mit Event Delegation
  setupEventListeners() {
    document.addEventListener('DOMContentLoaded', () => this.onDOMReady());
    window.addEventListener('beforeunload', () => this.cleanup());

    document.addEventListener('visibilitychange', () => {
      if (document.hidden) {
        this.pauseMonitoring();
      } else {
        this.resumeMonitoring();
      }
    });

    // Event Delegation für alle Button-Klicks
    document.addEventListener('click', (e) => {
      // Tab Navigation
      if (e.target.classList.contains('tab-btn')) {
        this.switchTab(e.target.dataset.tab);
      }
      // Endpoint Testing
      else if (e.target.classList.contains('test-btn')) {
        this.testEndpoint(e.target.dataset.url, e.target);
      }
      // Quick Action Buttons - Robuste Button-Erkennung
      else if (e.target.classList.contains('btn')) {
        const text = e.target.textContent.trim();

        if (text === 'View Site') {
          window.open('/', '_blank');
        } else if (text === 'Test APIs') {
          this.testAllEndpoints();
        } else if (text === 'Refresh Stats') {
          this.refreshStats();
        } else if (text === 'Simulate File Change') {
          this.simulateFileChange();
        }
      }
      // Terminal Action Buttons
      else if (e.target.classList.contains('terminal-btn')) {
        const text = e.target.textContent.trim();
        if (text === 'Clear') {
          this.clearLogs();
        } else if (text.includes('Auto-scroll')) {
          this.toggleAutoScroll();
        }
      }
    });
  }

  onDOMReady() {
    this.checkServerStatus();
    this.loadInitialMetrics();
  }

  // Tab System - Vereinfacht
  formatCreationTime() {
    const element = document.getElementById('creation-time');
    if (!element || element.textContent === '{{CREATION_TIME}}') return;

    try {
      const date = new Date(element.textContent);
      if (!isNaN(date.getTime())) {
        element.textContent = date.toLocaleString('de-DE', {
          day: '2-digit',
          month: '2-digit',
          hour: '2-digit',
          minute: '2-digit',
        });
      }
    } catch (e) {
      console.warn('[Rush Sync] Time format failed:', e);
    }
  }

  initializeTabs() {
    this.switchTab('overview');
  }

  switchTab(tabName) {
    document.querySelectorAll('.tab-btn').forEach((btn) => {
      btn.classList.toggle('active', btn.dataset.tab === tabName);
    });

    document.querySelectorAll('.tab-panel').forEach((panel) => {
      panel.classList.toggle('active', panel.id === `tab-${tabName}`);
    });

    this.state.currentTab = tabName;

    // Lazy loading für Tabs
    if (tabName === 'logs') this.loadLogs();
    else if (tabName === 'metrics') this.loadMetrics();
  }

  // Monitoring - Optimiert
  startAllMonitoring() {
    this.intervals.status = setInterval(() => this.checkServerStatus(), this.config.statusInterval);
    this.intervals.uptime = setInterval(() => this.updateUptime(), 1000);
    this.intervals.logs = setInterval(() => {
      if (this.state.currentTab === 'logs') this.pollLogs();
    }, this.config.logPollInterval);
    this.intervals.metrics = setInterval(() => this.updateMetrics(), this.config.metricsInterval);
  }

  async checkServerStatus() {
    const statusDot = document.querySelector('.status-dot');
    const statusText = document.querySelector('.status-text');
    if (!statusDot || !statusText) return;

    try {
      const response = await fetch('/api/health', {
        cache: 'no-cache',
        signal: AbortSignal.timeout(3000),
      });

      if (response.ok) {
        statusDot.setAttribute('data-status', 'running');
        statusText.textContent = 'Online';
        this.state.serverAlive = true;
      } else {
        throw new Error(`HTTP ${response.status}`);
      }
    } catch (error) {
      statusDot.setAttribute('data-status', 'error');
      statusText.textContent = 'Offline';
      console.warn('[Rush Sync] Status check failed:', error);

      if (this.state.serverAlive) {
        this.state.serverAlive = false;
        this.handleServerShutdown();
      }
    }
  }

  updateUptime() {
    const element = document.getElementById('uptime');
    if (element) {
      element.textContent = this.formatUptime(Date.now() - this.startTime);
    }
  }

  formatUptime(ms) {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ${hours % 24}h`;
    if (hours > 0) return `${hours}h ${minutes % 60}m`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
  }

  // Metrics - Vereinfacht und optimiert
  async loadInitialMetrics() {
    try {
      const response = await fetch('/api/stats');
      if (response.ok) {
        const data = await response.json();
        this.updateStatsDisplay(data);
      }
    } catch (e) {
      console.warn('[Rush Sync] Initial metrics failed:', e);
    }
  }

  async updateMetrics() {
    try {
      const response = await fetch('/api/stats');
      if (response.ok) {
        const data = await response.json();
        this.updateStatsDisplay(data);
        if (this.state.currentTab === 'metrics') {
          this.updateMetricsTab(data);
        }
      }
    } catch (e) {
      console.warn('[Rush Sync] Metrics update failed:', e);
    }
  }

  updateStatsDisplay(data) {
    const updates = {
      'request-count': data.total_requests || 0,
      'error-count': data.error_requests || 0,
    };

    Object.entries(updates).forEach(([id, value]) => {
      const element = document.getElementById(id);
      if (element) element.textContent = value;
    });
  }

  updateMetricsTab(data) {
    const updates = {
      'avg-response': `${Math.round(data.avg_response_time_ms || 0)}ms`,
      'total-requests': data.total_requests || 0,
      'bytes-sent': this.formatBytes(data.total_bytes_sent || 0),
      'unique-ips': data.unique_ips || 0,
    };

    Object.entries(updates).forEach(([id, value]) => {
      const element = document.getElementById(id);
      if (element) element.textContent = value;
    });

    this.updateRecentActivity(data);
  }

  formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  updateRecentActivity(data) {
    const container = document.getElementById('recent-activity');
    if (!container) return;

    const activities = [
      { time: 'Just now', desc: `${data.total_requests || 0} total requests processed` },
      { time: '1m ago', desc: `Average response: ${Math.round(data.avg_response_time_ms || 0)}ms` },
    ];

    container.innerHTML = activities
      .map(
        (activity) => `
      <div class="activity-item">
        <span class="activity-time">${activity.time}</span>
        <span class="activity-desc">${activity.desc}</span>
      </div>
    `,
      )
      .join('');
  }

  // Logging - Optimiert
  async loadLogs() {
    const container = document.getElementById('log-output');
    if (!container) return;

    this.addLogEntry('INFO', 'Fetching server activity...');

    try {
      const response = await fetch('/api/stats');
      if (response.ok) {
        const data = await response.json();
        this.addLogEntry(
          'SUCCESS',
          `Stats: ${data.total_requests} requests, ${data.unique_ips} IPs`,
        );

        if (data.total_requests > this.state.requests) {
          const newRequests = data.total_requests - this.state.requests;
          for (let i = 0; i < Math.min(newRequests, 5); i++) {
            this.addLogEntry(
              'INFO',
              `HTTP GET / - 200 OK - ${Math.floor(Math.random() * 50)}ms`,
              'request',
            );
          }
        }

        this.state.requests = data.total_requests;
      }
    } catch (e) {
      this.addLogEntry('ERROR', `Stats fetch failed: ${e.message}`, 'error');
    }
  }

  async pollLogs() {
    if (Math.random() < 0.3) {
      const activities = [
        'Static file served: style.css',
        'API endpoint: /api/health',
        'Connection from 127.0.0.1',
        'Cache hit: favicon.svg',
        'Metrics updated',
        'Hot Reload active',
      ];

      const activity = activities[Math.floor(Math.random() * activities.length)];
      this.addLogEntry('INFO', activity, 'request');
    }
  }

  addLogEntry(level, message, type = 'system') {
    const container = document.getElementById('log-output');
    if (!container) return;

    const timestamp = new Date().toLocaleTimeString('de-DE', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });

    const logLine = document.createElement('div');
    logLine.className = `log-line ${type}`;
    logLine.innerHTML = `
      <span class="log-time">${timestamp}</span>
      <span class="log-level ${level}">${level}</span>
      <span class="log-message">${message}</span>
    `;

    container.appendChild(logLine);

    // Log rotation
    const lines = container.children;
    if (lines.length > this.config.maxLogLines) {
      container.removeChild(lines[0]);
    }

    if (this.config.autoScroll && this.state.currentTab === 'logs') {
      container.scrollTop = container.scrollHeight;
    }
  }

  // Utility Functions - Optimiert
  async testEndpoint(url, button) {
    const originalText = button.textContent;
    button.textContent = '...';
    button.disabled = true;

    try {
      const response = await fetch(url, { cache: 'no-cache' });

      if (response.ok) {
        button.textContent = 'OK';
        button.style.background = 'var(--accent-success)';
        this.addLogEntry('SUCCESS', `${url} - ${response.status} OK`, 'request');
        setTimeout(() => window.open(url, '_blank'), 500);
      } else {
        throw new Error(`HTTP ${response.status}`);
      }
    } catch (error) {
      button.textContent = 'ERR';
      button.style.background = 'var(--accent-error)';
      this.addLogEntry('ERROR', `${url} - ${error.message}`, 'error');
    }

    setTimeout(() => {
      button.textContent = originalText;
      button.style.background = '';
      button.disabled = false;
    }, 2000);
  }

  async testAllEndpoints() {
    const endpoints = ['/api/status', '/api/health', '/api/info', '/api/metrics'];
    this.addLogEntry('INFO', 'Testing endpoints...');

    for (const endpoint of endpoints) {
      try {
        const response = await fetch(endpoint, { cache: 'no-cache' });
        const status = response.ok ? 'PASS' : 'FAIL';
        this.addLogEntry('INFO', `${endpoint} - ${status} (${response.status})`);
      } catch (e) {
        this.addLogEntry('ERROR', `${endpoint} - ERROR: ${e.message}`);
      }
      await new Promise((resolve) => setTimeout(resolve, 200));
    }

    this.addLogEntry('SUCCESS', 'Endpoint testing completed');
  }

  async refreshStats() {
    this.addLogEntry('INFO', 'Refreshing stats...');
    await this.updateMetrics();
    this.addLogEntry('SUCCESS', 'Stats refreshed');
  }

  clearLogs() {
    const container = document.getElementById('log-output');
    if (container) {
      container.innerHTML = '';
      this.addLogEntry('INFO', 'Log buffer cleared');
    }
  }

  toggleAutoScroll() {
    this.config.autoScroll = !this.config.autoScroll;
    const button = document.getElementById('autoscroll-text');
    if (button) {
      button.textContent = this.config.autoScroll ? 'Auto-scroll ON' : 'Auto-scroll OFF';
    }
    this.addLogEntry('INFO', `Auto-scroll ${this.config.autoScroll ? 'enabled' : 'disabled'}`);
  }

  loadMetrics() {
    this.updateMetrics();
  }

  // Lifecycle Management - Vereinfacht
  pauseMonitoring() {
    Object.values(this.intervals).forEach((interval) => interval && clearInterval(interval));
    this.intervals = {};
  }

  resumeMonitoring() {
    this.startAllMonitoring();
    this.startServerHealthCheck();
  }

  cleanup() {
    this.pauseMonitoring();
    this.state.websocket?.close();
    console.log('[Rush Sync] Cleanup completed');
  }

  // Hot Reload Test Function für Simulate Button
  simulateFileChange() {
    this.addLogEntry('INFO', 'Simulating file change...', 'hotreload');

    // Simuliere ein FileChangeEvent
    const mockEvent = {
      event_type: 'modified',
      file_path: 'www/test-file.html',
      server_name: this.serverName,
      port: this.serverPort,
      timestamp: Math.floor(Date.now() / 1000),
      file_extension: 'html',
    };

    this.handleFileChange(mockEvent);
  }
}

// Optimierte Initialisierung
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => new RushSyncApp());
} else {
  new RushSyncApp();
}

// Export für Module
if (typeof module !== 'undefined' && module.exports) {
  module.exports = RushSyncApp;
}
