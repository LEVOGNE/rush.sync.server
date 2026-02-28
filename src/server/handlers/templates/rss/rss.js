/**
 * Rush Sync - Entry Point
 * Dashboard on admin pages, Hot-Reload only on public/user pages
 */

window.RUSH_CONFIG = {
  serverName: '{{SERVER_NAME}}',
  serverPort: '{{PORT}}',
  proxyHttpPort: '{{PROXY_PORT}}',
  proxyHttpsPort: '{{PROXY_HTTPS_PORT}}',
};

function isDashboardPage() {
  return !!document.querySelector('.tab-btn, .status-dot, [data-tab]');
}

function initHotReload() {
  var protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';
  var wsUrl = protocol + '//' + location.host + '/ws/hot-reload';
  console.log('[Rush Sync] Hot-Reload connecting to', wsUrl);
  var ws = new WebSocket(wsUrl);

  ws.onopen = function () {
    console.log('[Rush Sync] Hot-Reload WebSocket connected');
  };

  ws.onmessage = function (event) {
    console.log('[Rush Sync] File changed:', event.data);
    try {
      var data = JSON.parse(event.data);
      if (data.event_type) {
        location.reload();
      }
    } catch (e) {
      console.error('[Rush Sync] Parse error:', e);
    }
  };

  ws.onerror = function (err) {
    console.error('[Rush Sync] WebSocket error:', err);
  };

  ws.onclose = function () {
    console.log('[Rush Sync] WebSocket closed, reconnecting in 3s...');
    setTimeout(initHotReload, 3000);
  };
}

function initDashboard() {
  import('/.rss/js/rush-app.js')
    .then(function (module) {
      new module.RushSyncApp();
      console.log('[Rush Sync] Dashboard loaded successfully');
    })
    .catch(function (error) {
      console.error('[Rush Sync] Dashboard failed to load:', error);
      document.body.innerHTML =
        '<div style="position:fixed;top:50%;left:50%;transform:translate(-50%,-50%);' +
        'background:#ff4757;color:white;padding:20px;border-radius:8px;' +
        'font-family:monospace;text-align:center;">' +
        '<h2>Dashboard Load Error</h2>' +
        '<p>Check console for details</p>' +
        '<p><strong>' + error.message + '</strong></p></div>';
    });
}

function init() {
  console.log('[Rush Sync] init — dashboard:', isDashboardPage());
  if (isDashboardPage()) {
    initDashboard();
  } else {
    initHotReload();
  }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
