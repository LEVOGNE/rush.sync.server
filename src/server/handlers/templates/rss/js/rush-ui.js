/**
 * Rush Sync UI Manager
 * DOM Manipulation, Tab System, WebSocket Management, Hot Reload, Notifications
 */

export class UIManager {
  constructor(config, state) {
    this.config = config;
    this.state = state;

    // WebSocket state
    this.websocket = null;
    this.websocketConnected = false;
    this.reconnectAttempts = 0;

    // UI state
    this.autoScroll = true;

    // Callbacks to App Controller
    this.onTabSwitch = null;
    this.onFileChange = null;
    this.onAction = null;

    // API Manager reference (set later)
    this.apiManager = null;
  }

  setApiManager(apiManager) {
    this.apiManager = apiManager;
  }

  // ===== TAB SYSTEM =====

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
    this.onTabSwitch?.(tabName);
  }

  // ===== DISPLAY UPDATES =====

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

  // ===== LOG DISPLAY =====

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

    if (this.autoScroll && this.state.currentTab === 'logs') {
      container.scrollTop = container.scrollHeight;
    }
  }

  displayLogs(entries) {
    const container = document.getElementById('log-output');
    if (!container) return;

    entries.forEach((entry) => {
      try {
        const logData = typeof entry === 'string' ? JSON.parse(entry) : entry;
        this.displayRealLogEntry(logData, container);
      } catch (e) {
        this.displayPlainLogEntry(entry, container);
      }
    });

    const lines = container.children;
    if (lines.length > this.config.maxLogLines) {
      const excess = lines.length - this.config.maxLogLines;
      for (let i = 0; i < excess; i++) {
        container.removeChild(lines[0]);
      }
    }

    if (this.autoScroll && this.state.currentTab === 'logs') {
      container.scrollTop = container.scrollHeight;
    }
  }

  displayRealLogEntry(logData, container) {
    const logLine = document.createElement('div');
    const eventType = this.apiManager.getLogEventType(logData);
    logLine.className = `log-line ${eventType}`;

    const timestamp = this.apiManager.formatLogTimestamp(logData.timestamp);
    const level = this.apiManager.getLogLevel(logData);
    const message = this.apiManager.formatLogMessage(logData);

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

  clearLogs() {
    const container = document.getElementById('log-output');
    if (container) {
      container.innerHTML = '';
      this.addLogEntry('INFO', 'Log buffer cleared');
    }
  }

  toggleAutoScroll() {
    this.autoScroll = !this.autoScroll;
    const button = document.getElementById('autoscroll-text');
    if (button) {
      button.textContent = this.autoScroll ? 'Auto-scroll ON' : 'Auto-scroll OFF';
    }
    this.addLogEntry('INFO', `Auto-scroll ${this.autoScroll ? 'enabled' : 'disabled'}`);
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
    a.download = `${this.config.serverName}-${this.config.serverPort}-logs.txt`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);

    this.addLogEntry('INFO', 'Logs downloaded', 'system');
  }

  // ===== WEBSOCKET & HOT RELOAD =====

  initializeHotReload() {
    this.connectWebSocket();
  }

  connectWebSocket() {
    if (this.websocket?.readyState === WebSocket.OPEN) return;

    const wsUrl = `ws://127.0.0.1:${this.config.serverPort}/ws/hot-reload`;

    try {
      this.websocket = new WebSocket(wsUrl);

      this.websocket.onopen = () => {
        this.websocketConnected = true;
        this.reconnectAttempts = 0;
        this.updateHotReloadStatus('connected');
        console.log('[Hot Reload] Connected');
        this.addLogEntry('SUCCESS', 'Hot Reload connected', 'system');
      };

      this.websocket.onmessage = (event) => {
        try {
          const changeEvent = JSON.parse(event.data);
          this.onFileChange?.(changeEvent);
        } catch (e) {
          console.warn('[Hot Reload] Invalid message:', e);
        }
      };

      this.websocket.onclose = (event) => {
        this.websocketConnected = false;
        this.updateHotReloadStatus('disconnected');

        if (!event.wasClean && !this.state.shutdownNotified) {
          console.log('[Hot Reload] Reconnecting...');
          this.addLogEntry('WARN', 'Hot Reload reconnecting...', 'system');
          this.attemptReconnect();
        }
      };

      this.websocket.onerror = () => {
        this.updateHotReloadStatus('error');
        this.addLogEntry('ERROR', 'Hot Reload error', 'error');
      };
    } catch (error) {
      console.error('[Hot Reload] Connection failed:', error);
      this.updateHotReloadStatus('error');
      this.attemptReconnect();
    }
  }

  attemptReconnect() {
    if (this.state.shutdownNotified || this.reconnectAttempts >= this.config.maxReconnectAttempts)
      return;

    this.reconnectAttempts++;
    const delay = this.config.websocketReconnectDelay * this.reconnectAttempts;

    setTimeout(() => {
      if (!this.state.shutdownNotified) {
        console.log(
          `[Hot Reload] Attempt ${this.reconnectAttempts}/${this.config.maxReconnectAttempts}`,
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
      connected: 'Active',
      disconnected: 'Reconnecting',
      error: 'Error',
    };

    statusElement.textContent = statusText[status] || 'Unknown';
    statusElement.className = `hotreload-status ${status}`;

    if (wsStatusElement) {
      wsStatusElement.textContent =
        status === 'connected' ? 'Connected' : status === 'disconnected' ? 'Reconnecting' : 'Error';
      wsStatusElement.className = `status-value ${status}`;
    }

    const reconnectElement = document.getElementById('reconnect-attempts');
    if (reconnectElement) {
      reconnectElement.textContent = this.reconnectAttempts;
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

    while (container.children.length > 20) {
      container.removeChild(container.lastChild);
    }
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

  closeWebSocket() {
    if (this.websocket) {
      this.websocket.close();
      this.websocket = null;
    }
  }

  // ===== EVENT HANDLERS =====

  handleClick(event) {
    event.preventDefault();

    if (event.target.classList.contains('tab-btn')) {
      this.switchTab(event.target.dataset.tab);
    } else if (event.target.classList.contains('test-btn')) {
      if (this.apiManager) {
        this.apiManager.testEndpoint(event.target.dataset.url, event.target);
      }
    } else if (event.target.hasAttribute('data-action')) {
      const action = event.target.dataset.action;
      const data = event.target.dataset;

      console.log('[UI] Action:', action);

      // Handle UI-specific actions
      if (action === 'simulate-file-change') {
        // This will be handled by app controller
        this.onAction?.(action, data);
      } else {
        // Pass other actions to app controller
        this.onAction?.(action, data);
      }
    } else if (event.target.classList.contains('terminal-btn')) {
      this.handleTerminalButton(event.target);
    }
  }

  handleTerminalButton(button) {
    const text = button.textContent.trim();
    if (text === 'Clear') {
      this.clearLogs();
    } else if (text.includes('Auto-scroll')) {
      this.toggleAutoScroll();
    } else if (text === 'Download Log') {
      this.downloadLogs();
    }
  }

  // ===== NOTIFICATIONS =====

  showNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.style.cssText = `
      position: fixed;
      top: 20px;
      right: 20px;
      background: var(--bg-secondary);
      color: var(--text-primary);
      padding: 12px 16px;
      border-radius: 6px;
      border: 1px solid var(--border);
      font-size: 13px;
      z-index: 10000;
      animation: slideIn 0.3s ease-out;
    `;
    notification.textContent = message;

    document.body.appendChild(notification);
    setTimeout(() => notification.remove(), 3000);
  }

  showServerShutdownScreen() {
    document.body.innerHTML = `
      <div style="position: absolute;left: 50%;top: 50%;transform: translate(-50%, -50%);text-align: center;padding: 30px;font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;background: rgba(0, 212, 255, 0.25);color: #ffffff;display: flex;flex-direction: column;justify-content: center;align-items: center;border-radius: 11px;">
        <div style="border: 2px solid #00d4ff;padding: 30px;border-radius: 6px;max-width: 500px;background: #000000;">
          <h1 style="margin: 0 0 20px 0;color: #ff4757;font-size: 24px;font-weight: 600;text-transform: uppercase;">Server Stopped</h1>
          <p style="margin: 10px 0; font-size: 16px; color: #ffffff; line-height: 1.4">Server '<strong style="color: #00d4ff">${this.config.serverName}</strong>' on port <strong style="color: #00d4ff">${this.config.serverPort}</strong> stopped.</p>
          <p style="margin: 20px 0; font-size: 14px; color: #a0a6b1">Closing in <span id="countdown" style="color: #ff4757; font-weight: 600">3</span> seconds...</p>
          <button onclick="window.close()" style="margin-top: 20px;padding: 10px 20px;background: #00d4ff;color: #1a1d23;border: none;cursor: pointer;border-radius: 5px;font-size: 12px;font-weight: 600;text-transform: uppercase;">Close Tab Now</button>
        </div>
      </div>`;

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
    console.log('[UI] Closing tab');
    try {
      window.close();
      setTimeout(() => {
        if (!window.closed) {
          window.location.href = 'about:blank';
        }
      }, 500);
    } catch (e) {
      console.warn('[UI] Close failed, redirecting:', e);
      window.location.href = 'about:blank';
    }
  }

  // ===== UTILITIES =====

  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // ===== CLEANUP =====

  cleanup() {
    this.closeWebSocket();
    console.log('[UI] Cleanup completed');
  }
}
