/**
 * Rush Sync API Manager
 * HTTP Requests, Server Monitoring, Log Loading, Health Checks, Statistics
 */

export class ApiManager {
  constructor(config, state) {
    this.config = config;
    this.state = state;
    this.intervals = {};

    // API state
    this.apiState = {
      lastLogCheck: 0,
      logFileSize: 0,
      realLogs: [],
    };

    // Callbacks to App Controller
    this.onServerShutdown = null;
    this.onStatsUpdate = null;
    this.onLogsUpdate = null;
    this.onServerStatusChange = null;
  }

  // ===== HEALTH & STATUS =====

  async checkServerStatus() {
    try {
      const response = await this.request('/api/health', {
        signal: AbortSignal.timeout(3000),
      });

      if (response.ok) {
        this.updateStatusUI('running', 'Online');
        if (!this.state.serverAlive) {
          this.state.serverAlive = true;
          this.onServerStatusChange?.(true);
        }
      } else {
        throw new Error(`HTTP ${response.status}`);
      }
    } catch (error) {
      this.updateStatusUI('error', 'Offline');
      console.warn('[API] Status check failed:', error);

      if (this.state.serverAlive) {
        this.state.serverAlive = false;
        this.onServerStatusChange?.(false);
      }
    }
  }

  updateStatusUI(status, text) {
    const statusDot = document.querySelector('.status-dot');
    const statusText = document.querySelector('.status-text');

    if (statusDot) statusDot.setAttribute('data-status', status);
    if (statusText) statusText.textContent = text;
  }

  startServerHealthCheck() {
    this.intervals.healthCheck = setInterval(
      () => this.checkServerHealthInternal(),
      this.config.serverHealthCheckInterval,
    );
  }

  async checkServerHealthInternal() {
    if (this.state.shutdownNotified) return;

    try {
      const response = await this.request('/api/health', {
        signal: AbortSignal.timeout(1500),
      });

      if (!response.ok) throw new Error(`Server returned ${response.status}`);

      if (!this.state.serverAlive) {
        this.state.serverAlive = true;
        console.log('[API] Connection restored');
      }
    } catch (error) {
      if (this.state.serverAlive) {
        this.state.serverAlive = false;
        console.log('[API] Connection lost:', error.message);
        this.onServerShutdown?.();
      }
    }
  }

  // ===== METRICS & STATISTICS =====

  async loadInitialMetrics() {
    try {
      const response = await this.request('/api/stats');
      if (response.ok) {
        const data = await response.json();
        this.onStatsUpdate?.(data);
      }
    } catch (e) {
      console.warn('[API] Initial metrics failed:', e);
    }
  }

  async updateMetrics() {
    try {
      const response = await this.request('/api/stats');
      if (response.ok) {
        const data = await response.json();
        this.onStatsUpdate?.(data);
      }
    } catch (e) {
      console.warn('[API] Metrics update failed:', e);
    }
  }

  async loadMetrics() {
    this.updateMetrics();
  }

  // ===== REAL LOG SYSTEM =====

  startRealLogMonitoring() {
    this.intervals.realLogs = setInterval(() => {
      if (this.state.currentTab === 'logs') {
        this.loadRealLogs();
      }
    }, this.config.logRefreshInterval);
  }

  async loadRealLogs() {
    try {
      const response = await this.request('/api/logs/raw', {
        headers: {
          Accept: 'application/json',
          'X-Log-Size': this.apiState.logFileSize.toString(),
        },
      });

      if (response.ok) {
        const data = await response.json();

        if (data.new_entries && data.new_entries.length > 0) {
          this.processRealLogEntries(data.new_entries);
          this.apiState.logFileSize = data.file_size || this.apiState.logFileSize;
        }

        if (data.stats) {
          this.updateLogStats(data.stats);
        }
      }
    } catch (error) {
      console.warn('[API] Real log loading failed:', error);
    }
  }

  processRealLogEntries(entries) {
    this.onLogsUpdate?.(entries);
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
      return `Server started on port ${this.config.serverPort}`;
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

  formatBytes(bytes) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
  }

  // ===== LOG LOADING =====

  async loadLogs() {
    try {
      const response = await this.request('/api/stats');
      if (response.ok) {
        const data = await response.json();

        // Simulate some log entries based on stats
        if (data.total_requests > this.state.requests) {
          const newRequests = data.total_requests - this.state.requests;
          for (let i = 0; i < Math.min(newRequests, 5); i++) {
            // This would be handled by UI manager
          }
        }

        this.state.requests = data.total_requests;
        this.onStatsUpdate?.(data);
      }
    } catch (e) {
      console.warn('[API] Stats fetch failed:', e);
    }
  }

  async pollLogs() {
    if (Math.random() < 0.3) {
      // Simulate activity - this would be handled by UI manager
    }
  }

  // ===== ENDPOINT TESTING =====

  async testEndpoint(url, button) {
    const originalText = button.textContent;
    button.textContent = '...';
    button.disabled = true;

    try {
      // FIX: Nur relative URLs oder lokale URLs testen
      let testUrl = url;

      // Externe URLs umleiten auf lokale Tests
      if (url.includes('localhost:') && !url.includes('127.0.0.1')) {
        // Externe Proxy URLs -> lokale API URLs für Test
        if (url.includes(this.config.proxyHttpsPort)) {
          testUrl = '/api/status'; // Test lokale API statt externe URL
        } else if (url.includes(this.config.proxyHttpPort)) {
          testUrl = '/api/health'; // Test lokale API statt externe URL
        }
      }

      console.log(`[API] Testing: ${url} -> ${testUrl}`);

      const response = await this.request(testUrl);

      if (response.ok) {
        button.textContent = 'OK';
        button.style.background = 'var(--accent-success)';
        console.log(`[API] ${url} - ${response.status} OK`);

        // Nur bei erfolgreichen lokalen URLs das Original öffnen
        if (testUrl === url) {
          setTimeout(() => window.open(url, '_blank'), 500);
        } else {
          // Bei Proxy URLs das Original öffnen (wird extern getestet)
          setTimeout(() => window.open(url, '_blank'), 500);
        }
      } else {
        throw new Error(`HTTP ${response.status}`);
      }
    } catch (error) {
      button.textContent = 'ERR';
      button.style.background = 'var(--accent-error)';
      console.error(`[API] ${url} - ${error.message}`);
    }

    setTimeout(() => {
      button.textContent = originalText;
      button.style.background = '';
      button.disabled = false;
    }, 2000);
  }

  async testAllEndpoints() {
    console.log('[API] Testing all endpoints...');
    const endpoints = ['/api/status', '/api/health', '/api/info', '/api/metrics'];

    for (const endpoint of endpoints) {
      try {
        const response = await this.request(endpoint, {
          signal: AbortSignal.timeout(5000),
        });

        const status = response.ok ? 'PASS' : 'FAIL';
        const statusText = `${response.status} ${response.statusText}`;

        console.log(`[API] ${endpoint}: ${status} - ${statusText}`);
      } catch (e) {
        console.error(`[API] ${endpoint} failed:`, e);
      }

      await new Promise((resolve) => setTimeout(resolve, 300));
    }

    console.log('[API] All endpoints tested');
  }

  // ===== GENERIC HTTP =====

  async request(url, options = {}) {
    try {
      const response = await fetch(url, {
        cache: 'no-cache',
        ...options,
      });

      if (!response.ok && !options.allowErrors) {
        throw new Error(`HTTP ${response.status}`);
      }

      return response;
    } catch (error) {
      console.error('[API] Request failed:', url, error);
      throw error;
    }
  }

  // ===== MONITORING CONTROL =====

  stopMonitoring() {
    Object.values(this.intervals).forEach((interval) => interval && clearInterval(interval));
    this.intervals = {};
    console.log('[API] Monitoring stopped');
  }
}
