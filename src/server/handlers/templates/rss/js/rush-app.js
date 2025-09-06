/**
 * Rush Sync App Controller
 * Zentrale Koordination, Configuration, State Management
 */

import { ApiManager } from '/.rss/js/rush-api.js';
import { UIManager } from '/.rss/js/rush-ui.js';

export class RushSyncApp {
  constructor() {
    // Template-Replacement prüfen und korrigieren
    this.config = this.parseAndValidateConfig();

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
      lastReloadTime: 0,
      lastLogCheck: 0,
      realLogs: [],
      logFileSize: 0,
    };

    this.intervals = {};
    this.startTime = Date.now();

    // Initialize Managers
    this.api = new ApiManager(this.config, this.state);
    this.ui = new UIManager(this.config, this.state);

    // Setup inter-module communication
    this.setupCallbacks();

    this.init();
  }

  parseAndValidateConfig() {
    const windowConfig = window.RUSH_CONFIG || {};

    // Template-Replacement Detection und Korrektur
    const serverName = this.validateTemplateValue(windowConfig.serverName, 'rss-001');
    const serverPort = this.validateTemplateValue(windowConfig.serverPort, '8000');
    const proxyHttpPort = this.validateTemplateValue(windowConfig.proxyHttpPort, '3000');
    const proxyHttpsPort = this.validateTemplateValue(windowConfig.proxyHttpsPort, '3443');

    const config = {
      statusInterval: 5000,
      logPollInterval: 3000,
      metricsInterval: 10000,
      maxLogLines: 1000,
      autoScroll: true,
      serverHealthCheckInterval: 2000,
      websocketReconnectDelay: 1000,
      maxReconnectAttempts: 10,
      hotReloadDebounce: 250,
      logRefreshInterval: 2000,

      // Validated template values
      serverName,
      serverPort,
      proxyHttpPort,
      proxyHttpsPort,
    };

    // Debug-Ausgabe
    console.log('[App] Template validation:', {
      raw: windowConfig,
      validated: {
        serverName: config.serverName,
        serverPort: config.serverPort,
        proxyHttpPort: config.proxyHttpPort,
        proxyHttpsPort: config.proxyHttpsPort,
      },
    });

    return config;
  }

  validateTemplateValue(value, fallback) {
    // Prüfe auf nicht-ersetzte Template-Platzhalter
    if (!value || typeof value !== 'string' || value.includes('{{') || value.includes('}}')) {
      console.warn(
        '[App] Template replacement failed for value:',
        value,
        'using fallback:',
        fallback,
      );
      return fallback;
    }
    return value;
  }

  setupCallbacks() {
    // API → App callbacks
    this.api.onServerShutdown = () => this.handleServerShutdown();
    this.api.onStatsUpdate = (data) => this.handleStatsUpdate(data);
    this.api.onLogsUpdate = (logs) => this.handleLogsUpdate(logs);
    this.api.onServerStatusChange = (alive) => this.handleServerStatusChange(alive);

    // UI → App callbacks
    this.ui.onTabSwitch = (tab) => this.handleTabSwitch(tab);
    this.ui.onFileChange = (event) => this.handleFileChange(event);
    this.ui.onAction = (action, data) => this.handleUIAction(action, data);

    // App → UI data flow
    this.ui.setApiManager(this.api);
  }

  init() {
    console.log('[Rush Sync] Initializing with validated config:', {
      server: this.config.serverName,
      port: this.config.serverPort,
      proxyHttp: this.config.proxyHttpPort,
      proxyHttps: this.config.proxyHttpsPort,
    });

    this.setupEventListeners();
    this.ui.initializeTabs();
    this.formatCreationTime();
    this.startAllMonitoring();
    this.ui.initializeHotReload();
    this.api.startRealLogMonitoring();

    // Global exposure for debugging
    window.RushApp = this;

    console.log('[Rush Sync] Dashboard initialized with Enhanced Logging');
    this.ui.addLogEntry(
      'INFO',
      `Enhanced Dashboard loaded - Proxy HTTP:${this.config.proxyHttpPort} HTTPS:${this.config.proxyHttpsPort}`,
    );
  }

  // ===== EVENT COORDINATION =====

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

    // Delegate all clicks to UI manager
    document.addEventListener('click', (e) => this.ui.handleClick(e));
  }

  onDOMReady() {
    this.api.checkServerStatus();
    this.api.loadInitialMetrics();
  }

  // ===== CALLBACK HANDLERS =====

  handleTabSwitch(tab) {
    this.state.currentTab = tab;

    if (tab === 'logs') {
      this.api.loadLogs();
      this.api.loadRealLogs();
    } else if (tab === 'metrics') {
      this.api.loadMetrics();
    }
  }

  handleFileChange(changeEvent) {
    const now = Date.now();
    if (now - this.state.lastReloadTime < this.config.hotReloadDebounce) return;

    console.log('[Hot Reload] File change:', changeEvent);

    const fileName = changeEvent.file_path.split('/').pop() || 'unknown';
    this.ui.addLogEntry('INFO', `File ${changeEvent.event_type}: ${fileName}`, 'hotreload');

    this.ui.addFileChangeToLog(changeEvent, fileName);

    if (this.shouldTriggerReload(changeEvent)) {
      this.state.lastReloadTime = now;
      this.ui.triggerPageReload(fileName);
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

  handleStatsUpdate(data) {
    this.ui.updateStatsDisplay(data);
    if (this.state.currentTab === 'metrics') {
      this.ui.updateMetricsTab(data);
    }
  }

  handleLogsUpdate(logs) {
    this.state.realLogs = logs;
    if (this.state.currentTab === 'logs') {
      this.ui.displayLogs(logs);
    }
  }

  handleServerStatusChange(alive) {
    this.state.serverAlive = alive;
    if (!alive && !this.state.shutdownNotified) {
      this.handleServerShutdown();
    }
  }

  handleUIAction(action, data) {
    console.log('[App] Action:', action);

    switch (action) {
      case 'view-http':
        window.open(`http://127.0.0.1:${this.config.serverPort}`, '_blank');
        break;
      case 'view-proxy':
        // Validierte Config verwenden
        window.open(
          `https://${this.config.serverName}.localhost:${this.config.proxyHttpsPort}`,
          '_blank',
        );
        break;
      case 'view-proxy-http':
        window.open(
          `http://${this.config.serverName}.localhost:${this.config.proxyHttpPort}`,
          '_blank',
        );
        break;
      case 'test-apis':
        this.api.testAllEndpoints();
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

  // ===== MONITORING =====

  startAllMonitoring() {
    this.intervals.status = setInterval(
      () => this.api.checkServerStatus(),
      this.config.statusInterval,
    );
    this.intervals.uptime = setInterval(() => this.updateUptime(), 1000);
    this.intervals.logs = setInterval(() => {
      if (this.state.currentTab === 'logs') this.api.pollLogs();
    }, this.config.logPollInterval);
    this.intervals.metrics = setInterval(
      () => this.api.updateMetrics(),
      this.config.metricsInterval,
    );
    this.api.startServerHealthCheck();
  }

  pauseMonitoring() {
    Object.values(this.intervals).forEach((interval) => interval && clearInterval(interval));
    this.intervals = {};
  }

  resumeMonitoring() {
    this.startAllMonitoring();
    this.api.startServerHealthCheck();
  }

  // ===== SERVER LIFECYCLE =====

  handleServerShutdown() {
    if (this.state.shutdownNotified) return;

    this.state.shutdownNotified = true;
    console.log('[Rush Sync] Server shutdown detected');

    this.ui.closeWebSocket();
    this.cleanup();
    this.ui.showServerShutdownScreen();
  }

  // ===== UTILITIES =====

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

  async refreshStats() {
    console.log('[Rush Sync] Refreshing stats...');
    this.ui.addLogEntry('INFO', 'Refreshing all stats...');

    try {
      await this.api.updateMetrics();
      await this.api.checkServerStatus();

      if (this.state.currentTab === 'logs') {
        this.api.loadLogs();
        this.api.loadRealLogs();
      }

      this.ui.addLogEntry('SUCCESS', 'Stats refreshed successfully');
      console.log('[Rush Sync] Stats refresh completed');
    } catch (error) {
      this.ui.addLogEntry('ERROR', `Stats refresh failed: ${error.message}`);
      console.error('[Rush Sync] Stats refresh failed:', error);
    }
  }

  simulateFileChange() {
    this.ui.addLogEntry('INFO', 'Simulating file change...', 'hotreload');

    const mockEvent = {
      event_type: 'modified',
      file_path: 'www/test-file.html',
      server_name: this.config.serverName,
      port: this.config.serverPort,
      timestamp: Math.floor(Date.now() / 1000),
      file_extension: 'html',
    };

    this.handleFileChange(mockEvent);
  }

  // ===== CLEANUP =====

  cleanup() {
    this.pauseMonitoring();
    this.api.stopMonitoring();
    this.ui.cleanup();
    console.log('[Rush Sync] Cleanup completed');
  }
}
