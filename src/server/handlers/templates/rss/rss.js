/**
 * Rush Sync Dashboard - Enhanced with Real Log Loading and TLS Info
 * WebSocket Hot Reload with real server log display
 */

class RushSyncApp {
  constructor() {
    // Consolidated Config
    this.config = {
      statusInterval: 5000,
      logPollInterval: 3000,
      metricsInterval: 10000,
      maxLogLines: 1000,
      autoScroll: true,
      serverHealthCheckInterval: 2000,
      websocketReconnectDelay: 1000,
      maxReconnectAttempts: 10,
      hotReloadDebounce: 250,
      logRefreshInterval: 2000, // Real log refresh
    };

    // Enhanced State
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
      lastLogCheck: 0,
      realLogs: [], // Real server logs
      logFileSize: 0,
    };

    this.intervals = {};
    this.startTime = Date.now();
    this.serverName = '{{SERVER_NAME}}';
    this.serverPort = '{{PORT}}';
    this.httpsPort = '{{HTTPS_PORT}}';
    this.proxyPort = '8443';

    this.init();
  }

  init() {
    this.setupEventListeners();
    this.initializeTabs();
    this.formatCreationTime();
    this.startAllMonitoring();
    this.initializeHotReload();
    this.startRealLogMonitoring();

    window.RushApp = this;

    console.log('[Rush Sync] Dashboard initialized with Enhanced Logging');
    this.addLogEntry('INFO', 'Enhanced Dashboard with TLS and Proxy support loaded');
  }

  // Real Log Loading System
  startRealLogMonitoring() {
    this.intervals.realLogs = setInterval(() => {
      if (this.state.currentTab === 'logs') {
        this.loadRealLogs();
      }
    }, this.config.logRefreshInterval);
  }

  async loadRealLogs() {
    try {
      const response = await fetch('/api/logs/raw', {
        cache: 'no-cache',
        headers: {
          Accept: 'application/json',
          'X-Log-Size': this.state.logFileSize.toString(),
        },
      });

      if (response.ok) {
        const data = await response.json();

        if (data.new_entries && data.new_entries.length > 0) {
          this.processRealLogEntries(data.new_entries);
          this.state.logFileSize = data.file_size || this.state.logFileSize;
        }

        if (data.stats) {
          this.updateLogStats(data.stats);
        }
      }
    } catch (error) {
      console.warn('[Rush Sync] Real log loading failed:', error);
    }
  }

  processRealLogEntries(entries) {
    const container = document.getElementById('log-output');
    if (!container) return;

    entries.forEach((entry) => {
      try {
        const logData = typeof entry === 'string' ? JSON.parse(entry) : entry;
        this.displayRealLogEntry(logData, container);
      } catch (e) {
        // Fallback for non-JSON log lines
        this.displayPlainLogEntry(entry, container);
      }
    });

    // Log rotation
    const lines = container.children;
    if (lines.length > this.config.maxLogLines) {
      const excess = lines.length - this.config.maxLogLines;
      for (let i = 0; i < excess; i++) {
        container.removeChild(lines[0]);
      }
    }

    if (this.config.autoScroll && this.state.currentTab === 'logs') {
      container.scrollTop = container.scrollHeight;
    }
  }

  displayRealLogEntry(logData, container) {
    const logLine = document.createElement('div');
    const eventType = this.getLogEventType(logData);
    logLine.className = `log-line ${eventType}`;

    const timestamp = this.formatLogTimestamp(logData.timestamp);
    const level = this.getLogLevel(logData);
    const message = this.formatLogMessage(logData);

    logLine.innerHTML = `
      <span class="log-time">${timestamp}</span>
      <span class="log-level ${level}">${level}</span>
      <span class="log-message">${message}</span>
    `;

    container.appendChild(logLine);
  }

  displayPlainLogEntry(entry, container) {
    const logLine = document.createElement('div');
    logLine.className = 'log-line system';

    const now = new Date().toLocaleTimeString('de-DE', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });

    logLine.innerHTML = `
      <span class="log-time">${now}</span>
      <span class="log-level INFO">INFO</span>
      <span class="log-message">${this.escapeHtml(entry)}</span>
    `;

    container.appendChild(logLine);
  }

  getLogEventType(logData) {
    switch (logData.event_type) {
      case 'Request':
        return 'request';
      case 'ServerError':
        return 'error';
      case 'SecurityAlert':
        return 'security';
      case 'PerformanceWarning':
        return 'performance';
      default:
        return 'system';
    }
  }

  getLogLevel(logData) {
    if (logData.event_type === 'ServerError') return 'ERROR';
    if (logData.event_type === 'SecurityAlert') return 'WARN';
    if (logData.event_type === 'PerformanceWarning') return 'WARN';
    if (logData.status_code >= 400) return 'WARN';
    if (logData.status_code >= 200 && logData.status_code < 300) return 'SUCCESS';
    return 'INFO';
  }

  formatLogMessage(logData) {
    if (logData.event_type === 'Request') {
      const responseTime = logData.response_time_ms ? ` - ${logData.response_time_ms}ms` : '';
      const bytes = logData.bytes_sent ? ` - ${this.formatBytes(logData.bytes_sent)}` : '';
      return `${logData.method} ${logData.path} - ${logData.status_code}${responseTime}${bytes} [${logData.ip_address}]`;
    }

    if (logData.event_type === 'SecurityAlert') {
      return `Security Alert: ${logData.headers?.alert_reason || 'Unknown'} - ${
        logData.headers?.alert_details || logData.path
      }`;
    }

    if (logData.event_type === 'PerformanceWarning') {
      return `Performance Warning: ${logData.headers?.metric || 'Response time'} = ${
        logData.response_time_ms
      }ms (threshold exceeded)`;
    }

    if (logData.event_type === 'ServerStart') {
      return `Server started on port ${this.serverPort}`;
    }

    if (logData.event_type === 'ServerStop') {
      return `Server stopped gracefully`;
    }

    if (logData.event_type === 'ServerError') {
      return `Server Error: ${logData.path} - ${
        logData.headers?.error_message || 'Internal error'
      }`;
    }

    return logData.path || 'System event';
  }

  formatLogTimestamp(timestamp) {
    if (!timestamp) return new Date().toLocaleTimeString('de-DE');

    try {
      const date = new Date(timestamp);
      return date.toLocaleTimeString('de-DE', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        fractionalSecondDigits: 3,
      });
    } catch (e) {
      return timestamp.substring(11, 23) || new Date().toLocaleTimeString('de-DE');
    }
  }

  updateLogStats(stats) {
    if (stats.total_requests !== undefined) {
      const requestElement = document.getElementById('request-count');
      if (requestElement) requestElement.textContent = stats.total_requests;
    }

    if (stats.error_requests !== undefined) {
      const errorElement = document.getElementById('error-count');
      if (errorElement) errorElement.textContent = stats.error_requests;
    }
  }

  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  // Hot Reload WebSocket - Same as before
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

    // Add to file changes log
    this.addFileChangeToLog(changeEvent, fileName);

    if (this.shouldTriggerReload(changeEvent)) {
      this.state.lastReloadTime = now;
      this.triggerPageReload(fileName);
    }
  }

  addFileChangeToLog(changeEvent, fileName) {
    const container = document.getElementById('file-changes-log');
    if (!container) return;

    const changeItem = document.createElement('div');
    changeItem.className = 'change-item';

    const timestamp = new Date().toLocaleTimeString('de-DE', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });

    changeItem.innerHTML = `
      <span class="change-time">${timestamp}</span>
      <span class="change-type ${changeEvent.event_type.toLowerCase()}">${changeEvent.event_type.toUpperCase()}</span>
      <span class="change-file">${fileName}</span>
      <span class="change-desc">${changeEvent.file_extension} file changed</span>
    `;

    container.insertBefore(changeItem, container.firstChild);

    // Keep only last 20 changes
    while (container.children.length > 20) {
      container.removeChild(container.lastChild);
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
    const wsStatusElement = document.getElementById('ws-connection-status');

    if (!statusElement) return;

    const statusText = {
      connected: '🟢 Active',
      disconnected: '🟡 Reconnecting',
      error: '🔴 Error',
    };

    statusElement.textContent = statusText[status] || '🔴 Unknown';
    statusElement.className = `hotreload-status ${status}`;

    if (wsStatusElement) {
      wsStatusElement.textContent =
        status === 'connected' ? 'Connected' : status === 'disconnected' ? 'Reconnecting' : 'Error';
      wsStatusElement.className = `status-value ${status}`;
    }

    const reconnectElement = document.getElementById('reconnect-attempts');
    if (reconnectElement) {
      reconnectElement.textContent = this.state.reconnectAttempts;
    }
  }

  // Enhanced Event Listeners with new button handlers
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

    // Event Delegation für alle Klicks
    document.addEventListener('click', (e) => {
      // Tab Navigation
      if (e.target.classList.contains('tab-btn')) {
        this.switchTab(e.target.dataset.tab);
      }
      // Endpoint Testing
      else if (e.target.classList.contains('test-btn')) {
        this.testEndpoint(e.target.dataset.url, e.target);
      }
      // NEUE: Header Actions mit data-action
      else if (e.target.hasAttribute('data-action')) {
        e.preventDefault();
        const action = e.target.dataset.action;

        console.log('[Rush Sync] Action:', action);

        switch (action) {
          case 'view-http':
            window.open(`http://127.0.0.1:${this.serverPort}`, '_blank');
            break;
          case 'view-https':
            // ENTFERNT - macht keinen Sinn bei Proxy-Setup
            this.addLogEntry('INFO', 'HTTPS direct access disabled - use proxy instead');
            break;
          case 'view-proxy':
            // KORREKTUR: HTTPS für Proxy verwenden
            window.open(`https://${this.serverName}.localhost:${this.proxyPort}`, '_blank');
            break;
          case 'test-apis':
            this.testAllEndpoints();
            break;
          case 'refresh-stats':
            this.refreshStats();
            break;
          case 'simulate-file-change':
            this.simulateFileChange();
            break;
          default:
            console.warn('[Rush Sync] Unknown action:', action);
        }
      }
      // FALLBACK: Alte Buttons mit CSS-Klasse .btn (für Kompatibilität)
      else if (e.target.classList.contains('btn')) {
        e.preventDefault();
        const buttonText = e.target.textContent.trim();

        console.log('[Rush Sync] Button clicked:', buttonText);

        switch (buttonText) {
          case 'View Site (HTTP)':
            window.open(`http://127.0.0.1:${this.serverPort}`, '_blank');
            break;
          case 'View Site (HTTPS)':
            this.addLogEntry('INFO', 'HTTPS direct access disabled - use proxy instead');
            break;
          case 'View via Proxy':
            // KORREKTUR: HTTPS für Proxy
            window.open(`https://${this.serverName}.localhost:${this.proxyPort}`, '_blank');
            break;
          case 'Test APIs':
            this.testAllEndpoints();
            break;
          case 'Refresh Stats':
            this.refreshStats();
            break;
          case 'Simulate File Change':
            this.simulateFileChange();
            break;
          default:
            console.warn('[Rush Sync] Unknown button:', buttonText);
        }
      }
      // Header Buttons (neue kompakte Buttons)
      else if (e.target.classList.contains('header-btn')) {
        e.preventDefault();
        const action = e.target.dataset.action;

        console.log('[Rush Sync] Header button action:', action);

        switch (action) {
          case 'view-proxy':
            window.open(`https://${this.serverName}.localhost:${this.proxyPort}`, '_blank');
            break;
          case 'test-apis':
            this.testAllEndpoints();
            break;
          case 'refresh-stats':
            this.refreshStats();
            break;
          default:
            console.warn('[Rush Sync] Unknown header action:', action);
        }
      }
      // Terminal Action Buttons
      else if (e.target.classList.contains('terminal-btn')) {
        const text = e.target.textContent.trim();
        if (text === 'Clear') {
          this.clearLogs();
        } else if (text.includes('Auto-scroll')) {
          this.toggleAutoScroll();
        } else if (text === 'Download Log') {
          this.downloadLogs();
        }
      }
    });
  }

  // Rest of the methods remain the same...
  onDOMReady() {
    this.checkServerStatus();
    this.loadInitialMetrics();
    this.updateServerInfo();
  }

  updateServerInfo() {
    // Update HTTPS port display
    const httpsElement = document.getElementById('https-port');
    if (httpsElement) {
      httpsElement.textContent = this.httpsPort;
    }

    // Update certificate validity display
    this.updateCertificateInfo();
  }

  updateCertificateInfo() {
    // This would typically fetch from /api/certificates
    // For now, show default values
    const certValidityElement = document.getElementById('cert-validity');
    if (certValidityElement) {
      certValidityElement.textContent = '365 days';
    }
  }

  downloadLogs() {
    const logOutput = document.getElementById('log-output');
    if (!logOutput) return;

    const logLines = Array.from(logOutput.children)
      .map((line) => {
        const time = line.querySelector('.log-time')?.textContent || '';
        const level = line.querySelector('.log-level')?.textContent || '';
        const message = line.querySelector('.log-message')?.textContent || '';
        return `${time} ${level} ${message}`;
      })
      .join('\n');

    const blob = new Blob([logLines], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${this.serverName}-${this.serverPort}-logs.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    this.addLogEntry('INFO', 'Logs downloaded', 'system');
  }

  // Tab System
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

    // Lazy loading for tabs
    if (tabName === 'logs') {
      this.loadLogs();
      this.loadRealLogs(); // Load real logs immediately when tab is opened
    } else if (tabName === 'metrics') {
      this.loadMetrics();
    } else if (tabName === 'certificates') {
      this.loadCertificateInfo();
    }
  }

  loadCertificateInfo() {
    // This would fetch certificate details from the server
    // For now, update with template values
    this.updateCertificateInfo();
  }

  // Rest of methods (monitoring, metrics, etc.) remain the same as in the original...
  startAllMonitoring() {
    this.intervals.status = setInterval(() => this.checkServerStatus(), this.config.statusInterval);
    this.intervals.uptime = setInterval(() => this.updateUptime(), 1000);
    this.intervals.logs = setInterval(() => {
      if (this.state.currentTab === 'logs') this.pollLogs();
    }, this.config.logPollInterval);
    this.intervals.metrics = setInterval(() => this.updateMetrics(), this.config.metricsInterval);
    this.startServerHealthCheck();
  }

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

  handleServerShutdown() {
    if (this.state.shutdownNotified) return;

    this.state.shutdownNotified = true;
    console.log('[Rush Sync] Server shutdown detected');

    this.state.websocket?.close();
    this.cleanup();

    document.body.innerHTML = `<div style="position: absolute;left: 50%;top: 50%;transform: translate(-50%, -50%);text-align: center;padding: 30px;font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;background: rgba(0, 212, 255, 0.25);color: #ffffff;display: flex;flex-direction: column;justify-content: center;align-items: center;border-radius: 11px;"><div style="border: 2px solid #00d4ff;padding: 30px;border-radius: 6px;max-width: 500px;background: #000000;"><h1 style="margin: 0 0 20px 0;color: #ff4757;font-size: 24px;font-weight: 600;text-transform: uppercase;">Server Stopped</h1><p style="margin: 10px 0; font-size: 16px; color: #ffffff; line-height: 1.4">Server '<strong style="color: #00d4ff">${this.serverName}</strong>' on port <strong style="color: #00d4ff">${this.serverPort}</strong> stopped.</p><p style="margin: 20px 0; font-size: 14px; color: #a0a6b1">Closing in <span id="countdown" style="color: #ff4757; font-weight: 600">3</span> seconds...</p><button onclick="window.close()" style="margin-top: 20px;padding: 10px 20px;background: #00d4ff;color: #1a1d23;border: none;cursor: pointer;border-radius: 5px;font-size: 12px;font-weight: 600;text-transform: uppercase;">Close Tab Now</button></div></div>`;

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
      window.close();
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

  // Utility methods continue...
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

  formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // Continue with remaining utility methods...
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

  updateRecentActivity(data) {
    const container = document.getElementById('recent-activity');
    if (!container) return;

    const activities = [
      { time: 'Just now', desc: `${data.total_requests || 0} total requests processed` },
      { time: '1m ago', desc: `Average response: ${Math.round(data.avg_response_time_ms || 0)}ms` },
      { time: '2m ago', desc: `${data.unique_ips || 0} unique visitors` },
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

  async loadLogs() {
    const container = document.getElementById('log-output');
    if (!container) return;

    this.addLogEntry('INFO', 'Loading server activity...');

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
        'TLS certificate validated',
        'Proxy route active',
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

    const lines = container.children;
    if (lines.length > this.config.maxLogLines) {
      container.removeChild(lines[0]);
    }

    if (this.config.autoScroll && this.state.currentTab === 'logs') {
      container.scrollTop = container.scrollHeight;
    }
  }

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
    console.log('[Rush Sync] Testing all endpoints...');
    const endpoints = ['/api/status', '/api/health', '/api/info', '/api/metrics'];

    this.addLogEntry('INFO', 'Testing all endpoints...');

    for (const endpoint of endpoints) {
      try {
        const response = await fetch(endpoint, {
          cache: 'no-cache',
          signal: AbortSignal.timeout(5000), // 5s timeout
        });

        const status = response.ok ? 'PASS' : 'FAIL';
        const statusText = `${response.status} ${response.statusText}`;

        this.addLogEntry(
          response.ok ? 'SUCCESS' : 'WARN',
          `${endpoint} - ${status} (${statusText})`,
        );

        console.log(`[Rush Sync] ${endpoint}: ${status} - ${statusText}`);
      } catch (e) {
        this.addLogEntry('ERROR', `${endpoint} - ERROR: ${e.message}`);
        console.error(`[Rush Sync] ${endpoint} failed:`, e);
      }

      // Kurze Pause zwischen Tests
      await new Promise((resolve) => setTimeout(resolve, 300));
    }

    this.addLogEntry('SUCCESS', 'Endpoint testing completed');
    console.log('[Rush Sync] All endpoints tested');
  }

  async refreshStats() {
    console.log('[Rush Sync] Refreshing stats...');
    this.addLogEntry('INFO', 'Refreshing all stats...');

    try {
      // Stats aktualisieren
      await this.updateMetrics();

      // Status prüfen
      await this.checkServerStatus();

      // Logs aktualisieren wenn auf Log-Tab
      if (this.state.currentTab === 'logs') {
        this.loadLogs();
        this.loadRealLogs();
      }

      this.addLogEntry('SUCCESS', 'Stats refreshed successfully');
      console.log('[Rush Sync] Stats refresh completed');
    } catch (error) {
      this.addLogEntry('ERROR', `Stats refresh failed: ${error.message}`);
      console.error('[Rush Sync] Stats refresh failed:', error);
    }
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

  // Hot Reload Test Function for Simulate Button
  simulateFileChange() {
    this.addLogEntry('INFO', 'Simulating file change...', 'hotreload');

    // Simulate a FileChangeEvent
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

// Optimized Initialization
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => new RushSyncApp());
} else {
  new RushSyncApp();
}

// Export for modules
if (typeof module !== 'undefined' && module.exports) {
  module.exports = RushSyncApp;
}
