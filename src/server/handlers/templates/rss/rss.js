/**
 * Rush Sync Dashboard - Entry Point
 * Lädt alle Module und startet das Dashboard
 */

import { RushSyncApp } from '/.rss/js/rush-app.js';

// Global config injection from Rust template
window.RUSH_CONFIG = {
  serverName: '{{SERVER_NAME}}',
  serverPort: '{{PORT}}',
  proxyHttpPort: '{{PROXY_PORT}}',
  proxyHttpsPort: '{{PROXY_HTTPS_PORT}}',
};

// Initialize Dashboard
function initDashboard() {
  try {
    new RushSyncApp();
    console.log('[Rush Sync] Dashboard loaded successfully');
  } catch (error) {
    console.error('[Rush Sync] Dashboard failed to load:', error);

    // Fallback error display
    document.body.innerHTML = `
      <div style="position: fixed; top: 50%; left: 50%; transform: translate(-50%, -50%);
                  background: #ff4757; color: white; padding: 20px; border-radius: 8px;
                  font-family: monospace; text-align: center;">
        <h2>Dashboard Load Error</h2>
        <p>Check console for details</p>
        <p><strong>${error.message}</strong></p>
      </div>`;
  }
}

// Start when ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initDashboard);
} else {
  initDashboard();
}

// Export for debugging
if (typeof module !== 'undefined' && module.exports) {
  module.exports = RushSyncApp;
}
