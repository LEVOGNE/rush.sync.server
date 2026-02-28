// src/server/analytics.rs
//
// Lightweight in-memory analytics tracker with periodic file persistence.
// Filters out noise (health checks, bots, internal assets) and tracks
// meaningful page views, downloads, unique visitors, and subdomain stats.

use chrono::{Local, NaiveDate, TimeDelta};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, OnceLock, RwLock};

static ANALYTICS: OnceLock<Arc<RwLock<AnalyticsTracker>>> = OnceLock::new();

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AnalyticsTracker {
    days: HashMap<String, DayData>,
    hourly: VecDeque<HourBucket>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct DayData {
    total_views: u64,
    total_downloads: u64,
    unique_ips: HashSet<String>,
    page_counts: HashMap<String, u64>,
    download_counts: HashMap<String, u64>,
    subdomain_views: HashMap<String, u64>,
    subdomain_ips: HashMap<String, HashSet<String>>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct HourBucket {
    hour: String,
    views: u64,
    unique_ips: HashSet<String>,
}

/// Get or initialize the global analytics tracker.
/// On first call, loads persisted data from disk and starts periodic save.
pub fn get_analytics() -> &'static Arc<RwLock<AnalyticsTracker>> {
    ANALYTICS.get_or_init(|| {
        let tracker = load_from_file().unwrap_or_default();
        let arc = Arc::new(RwLock::new(tracker));

        let arc_clone = arc.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("analytics save runtime");
            rt.block_on(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
                    if let Ok(tracker) = arc_clone.read() {
                        if let Err(e) = save_to_file(&tracker) {
                            log::error!("Failed to save analytics: {}", e);
                        }
                    }
                }
            });
        });

        log::info!("Analytics tracker initialized");
        arc
    })
}

/// Track a single request. Called from proxy handler and server middleware.
/// Filters out non-trackable requests (health checks, bots, internal assets).
pub fn track_request(subdomain: &str, path: &str, ip: &str, user_agent: &str) {
    if !is_trackable_request(path, user_agent) {
        return;
    }

    let analytics = get_analytics();
    let mut tracker = match analytics.write() {
        Ok(t) => t,
        Err(_) => return,
    };

    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let hour_key = now.format("%Y-%m-%dT%H:00").to_string();
    let ip_hash = hash_ip(ip);
    let subdomain_key = if subdomain.is_empty() {
        "direct"
    } else {
        subdomain
    };

    let clean_path = path.split('?').next().unwrap_or(path);

    // Update day data
    let day = tracker.days.entry(date).or_default();
    day.total_views += 1;
    day.unique_ips.insert(ip_hash.clone());
    *day.page_counts.entry(clean_path.to_string()).or_default() += 1;
    *day.subdomain_views
        .entry(subdomain_key.to_string())
        .or_default() += 1;
    day.subdomain_ips
        .entry(subdomain_key.to_string())
        .or_default()
        .insert(ip_hash.clone());

    if is_download(clean_path) {
        day.total_downloads += 1;
        *day.download_counts
            .entry(clean_path.to_string())
            .or_default() += 1;
    }

    // Update hourly bucket
    if let Some(bucket) = tracker.hourly.back_mut() {
        if bucket.hour == hour_key {
            bucket.views += 1;
            bucket.unique_ips.insert(ip_hash);
            return;
        }
    }
    let mut ips = HashSet::new();
    ips.insert(ip_hash);
    tracker.hourly.push_back(HourBucket {
        hour: hour_key,
        views: 1,
        unique_ips: ips,
    });
    while tracker.hourly.len() > 48 {
        tracker.hourly.pop_front();
    }
}

fn is_trackable_request(path: &str, user_agent: &str) -> bool {
    let path_lower = path.to_lowercase();
    let clean = path_lower.split('?').next().unwrap_or(&path_lower);

    // Filter monitoring/internal endpoints
    if matches!(
        clean,
        "/api/health"
            | "/api/status"
            | "/api/metrics"
            | "/api/analytics"
            | "/api/analytics/dashboard"
            | "/api/logs"
            | "/api/logs/raw"
            | "/api/ping"
    ) {
        return false;
    }

    // Filter internal assets
    if clean.starts_with("/.rss/")
        || clean == "/rss.js"
        || clean.starts_with("/ws/")
        || clean.starts_with("/.well-known/")
        || clean == "/favicon.ico"
    {
        return false;
    }

    // Filter bots/crawlers
    let ua = user_agent.to_lowercase();
    if ua.contains("bot")
        || ua.contains("crawler")
        || ua.contains("spider")
        || ua.contains("curl")
        || ua.contains("wget")
        || ua.contains("python-requests")
        || ua.contains("go-http-client")
        || ua.contains("headlesschrome")
        || ua.contains("phantomjs")
    {
        return false;
    }

    true
}

fn is_download(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.ends_with(".zip")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".exe")
        || lower.ends_with(".dmg")
        || lower.ends_with(".deb")
        || lower.ends_with(".rpm")
        || lower.ends_with(".msi")
        || lower.ends_with(".pkg")
        || lower.ends_with(".appimage")
}

fn hash_ip(ip: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    ip.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Get analytics summary as JSON for the API endpoint.
pub fn get_summary() -> serde_json::Value {
    let analytics = get_analytics();
    let tracker = match analytics.read() {
        Ok(t) => t,
        Err(_) => return json!({"error": "lock poisoned"}),
    };

    let now = Local::now();
    let today = now.format("%Y-%m-%d").to_string();

    let today_data = build_period_summary(&tracker, &today, 1);
    let week_data = build_period_summary(&tracker, &today, 7);
    let month_data = build_period_summary(&tracker, &today, 30);

    let cutoff = (now - TimeDelta::hours(24))
        .format("%Y-%m-%dT%H:00")
        .to_string();
    let hourly: Vec<serde_json::Value> = tracker
        .hourly
        .iter()
        .filter(|b| b.hour >= cutoff)
        .map(|b| {
            json!({
                "hour": b.hour,
                "views": b.views,
                "unique": b.unique_ips.len()
            })
        })
        .collect();

    let by_subdomain = build_subdomain_summary(&tracker, &today, 7);

    json!({
        "today": today_data,
        "last_7_days": week_data,
        "last_30_days": month_data,
        "hourly_traffic": hourly,
        "by_subdomain": by_subdomain,
    })
}

fn build_period_summary(
    tracker: &AnalyticsTracker,
    today: &str,
    days: i64,
) -> serde_json::Value {
    let today_date = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .unwrap_or_else(|_| Local::now().date_naive());

    let mut total_views = 0u64;
    let mut total_downloads = 0u64;
    let mut all_ips: HashSet<String> = HashSet::new();
    let mut page_totals: HashMap<String, u64> = HashMap::new();
    let mut download_totals: HashMap<String, u64> = HashMap::new();

    for i in 0..days {
        let date = (today_date - TimeDelta::days(i))
            .format("%Y-%m-%d")
            .to_string();
        if let Some(day) = tracker.days.get(&date) {
            total_views += day.total_views;
            total_downloads += day.total_downloads;
            all_ips.extend(day.unique_ips.iter().cloned());
            for (path, count) in &day.page_counts {
                *page_totals.entry(path.clone()).or_default() += count;
            }
            for (file, count) in &day.download_counts {
                *download_totals.entry(file.clone()).or_default() += count;
            }
        }
    }

    let mut pages: Vec<_> = page_totals.into_iter().collect();
    pages.sort_by(|a, b| b.1.cmp(&a.1));
    let top_pages: Vec<serde_json::Value> = pages
        .into_iter()
        .take(10)
        .map(|(path, views)| json!({"path": path, "views": views}))
        .collect();

    let mut downloads: Vec<_> = download_totals.into_iter().collect();
    downloads.sort_by(|a, b| b.1.cmp(&a.1));
    let top_downloads: Vec<serde_json::Value> = downloads
        .into_iter()
        .take(10)
        .map(|(file, count)| json!({"file": file, "count": count}))
        .collect();

    json!({
        "page_views": total_views,
        "unique_visitors": all_ips.len(),
        "downloads": total_downloads,
        "top_pages": top_pages,
        "top_downloads": top_downloads,
    })
}

fn build_subdomain_summary(
    tracker: &AnalyticsTracker,
    today: &str,
    days: i64,
) -> serde_json::Value {
    let today_date = NaiveDate::parse_from_str(today, "%Y-%m-%d")
        .unwrap_or_else(|_| Local::now().date_naive());

    let mut views: HashMap<String, u64> = HashMap::new();
    let mut ips: HashMap<String, HashSet<String>> = HashMap::new();

    for i in 0..days {
        let date = (today_date - TimeDelta::days(i))
            .format("%Y-%m-%d")
            .to_string();
        if let Some(day) = tracker.days.get(&date) {
            for (sub, v) in &day.subdomain_views {
                *views.entry(sub.clone()).or_default() += v;
            }
            for (sub, ip_set) in &day.subdomain_ips {
                ips.entry(sub.clone())
                    .or_default()
                    .extend(ip_set.iter().cloned());
            }
        }
    }

    let mut map = serde_json::Map::new();
    for (sub, v) in &views {
        let unique = ips.get(sub).map(|s| s.len()).unwrap_or(0);
        map.insert(sub.clone(), json!({"views": v, "unique": unique}));
    }
    serde_json::Value::Object(map)
}

fn get_analytics_path() -> std::path::PathBuf {
    crate::core::helpers::get_base_dir()
        .map(|b| b.join(".rss").join("analytics.json"))
        .unwrap_or_else(|_| std::path::PathBuf::from(".rss/analytics.json"))
}

fn save_to_file(tracker: &AnalyticsTracker) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_analytics_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string(tracker)?;
    std::fs::write(&path, json)?;
    log::debug!("Analytics saved to {:?}", path);
    Ok(())
}

fn load_from_file() -> Option<AnalyticsTracker> {
    let path = get_analytics_path();
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save analytics to disk. Called during shutdown.
pub fn save_analytics_on_shutdown() {
    if let Some(analytics) = ANALYTICS.get() {
        if let Ok(mut tracker) = analytics.write() {
            prune_old_data(&mut tracker);
            if let Err(e) = save_to_file(&tracker) {
                log::error!("Failed to save analytics on shutdown: {}", e);
            } else {
                log::info!("Analytics saved on shutdown");
            }
        }
    }
}

fn prune_old_data(tracker: &mut AnalyticsTracker) {
    let cutoff = (Local::now() - TimeDelta::days(60))
        .format("%Y-%m-%d")
        .to_string();
    tracker
        .days
        .retain(|date, _| date.as_str() >= cutoff.as_str());
}

/// Dashboard HTML template. The placeholder `__ANALYTICS_DATA__` is replaced
/// with the current analytics JSON at render time.
pub const DASHBOARD_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"><meta name="viewport" content="width=device-width,initial-scale=1.0">
<title>Analytics - Rush Sync Server</title>
<link rel="icon" href="/.rss/favicon.svg" type="image/svg+xml">
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',sans-serif;background:#0a0a0f;color:#e4e4ef;min-height:100vh}
.container{max-width:1200px;margin:0 auto;padding:24px}
.header{display:flex;justify-content:space-between;align-items:center;margin-bottom:24px}
.header h1{font-size:24px;font-weight:700;letter-spacing:-0.5px}
.header h1 span{color:#6c63ff}
.back{color:#6c63ff;text-decoration:none;font-size:14px}
.cards{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px;margin-bottom:24px}
.card{background:#14141f;border:1px solid #2a2a3a;border-radius:12px;padding:20px}
.card .lbl{font-size:12px;color:#8888a0;text-transform:uppercase;letter-spacing:0.5px;margin-bottom:8px}
.card .val{font-size:36px;font-weight:700}
.card .val.purple{color:#6c63ff}
.card .val.green{color:#00d4aa}
.card .val.blue{color:#00a8ff}
.tabs{display:flex;gap:8px;margin-bottom:16px}
.tab{padding:8px 16px;border-radius:8px;background:#14141f;border:1px solid #2a2a3a;color:#8888a0;cursor:pointer;font-size:13px;transition:all 0.2s}
.tab:hover{border-color:#6c63ff}
.tab.active{background:#6c63ff;color:#fff;border-color:#6c63ff}
.section{background:#14141f;border:1px solid #2a2a3a;border-radius:12px;padding:20px;margin-bottom:16px}
.section h2{font-size:15px;margin-bottom:16px;font-weight:600;color:#c0c0d0}
.chart{display:flex;align-items:flex-end;gap:2px;height:140px;padding-bottom:24px;position:relative}
.bar-w{flex:1;display:flex;flex-direction:column;align-items:center;position:relative}
.bar{width:100%;background:linear-gradient(180deg,#6c63ff,#4a43cc);border-radius:3px 3px 0 0;min-height:2px;transition:height 0.3s;cursor:pointer}
.bar:hover{background:linear-gradient(180deg,#8b83ff,#6c63ff)}
.bar-lbl{font-size:8px;color:#8888a0;position:absolute;bottom:-20px;white-space:nowrap}
.tooltip{display:none;position:absolute;top:-30px;background:#2a2a3a;color:#e4e4ef;padding:4px 8px;border-radius:4px;font-size:11px;white-space:nowrap;z-index:10}
.bar:hover+.tooltip{display:block}
.grid{display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:16px}
@media(max-width:768px){.grid{grid-template-columns:1fr}.cards{grid-template-columns:1fr 1fr}}
table{width:100%;border-collapse:collapse}
th{text-align:left;font-size:11px;color:#8888a0;text-transform:uppercase;letter-spacing:0.5px;padding:8px 0;border-bottom:1px solid #2a2a3a}
td{padding:8px 0;font-size:13px;border-bottom:1px solid #1a1a2a}
td:last-child{text-align:right;font-weight:600;color:#6c63ff}
.sub-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(180px,1fr));gap:12px}
.sub-card{background:#1a1a2a;border-radius:8px;padding:16px}
.sub-card .name{font-weight:600;margin-bottom:4px;color:#e4e4ef}
.sub-card .stats{font-size:12px;color:#8888a0}
.empty{color:#555;font-style:italic;font-size:13px;padding:20px;text-align:center}
.footer{text-align:center;font-size:11px;color:#555;padding:16px}
</style>
</head>
<body>
<div class="container">
<div class="header"><h1>Analytics <span>Dashboard</span></h1><a href="/" class="back">&larr; Back</a></div>
<div class="cards" id="cards"></div>
<div class="tabs" id="tabs">
<div class="tab active" data-p="today">Today</div>
<div class="tab" data-p="last_7_days">7 Days</div>
<div class="tab" data-p="last_30_days">30 Days</div>
</div>
<div class="section"><h2>Hourly Traffic (Last 24h)</h2><div class="chart" id="chart"></div></div>
<div class="grid">
<div class="section"><h2>Top Pages</h2><div id="pages"></div></div>
<div class="section"><h2>Top Downloads</h2><div id="downloads"></div></div>
</div>
<div class="section"><h2>By Subdomain</h2><div class="sub-grid" id="subs"></div></div>
<div class="footer" id="foot">Loading...</div>
</div>
<script>
var D=__ANALYTICS_DATA__;
var P='today';
document.querySelectorAll('.tab').forEach(function(t){t.addEventListener('click',function(){document.querySelectorAll('.tab').forEach(function(x){x.classList.remove('active')});t.classList.add('active');P=t.dataset.p;render()})});
function render(){var p=D[P]||D.today||{};
document.getElementById('cards').innerHTML='<div class="card"><div class="lbl">Page Views</div><div class="val purple">'+fmt(p.page_views)+'</div></div>'+'<div class="card"><div class="lbl">Unique Visitors</div><div class="val green">'+fmt(p.unique_visitors)+'</div></div>'+'<div class="card"><div class="lbl">Downloads</div><div class="val blue">'+fmt(p.downloads)+'</div></div>';
var h=D.hourly_traffic||[];
if(h.length===0){document.getElementById('chart').innerHTML='<div class="empty">No hourly data yet</div>'}
else{var mx=Math.max.apply(null,h.map(function(x){return x.views}))||1;document.getElementById('chart').innerHTML=h.map(function(x){var pct=Math.max((x.views/mx)*100,2);var hr=(x.hour.split('T')[1]||'').replace(':00','h');return '<div class="bar-w"><div class="bar" style="height:'+pct+'%"></div><div class="tooltip">'+x.views+' views, '+x.unique+' unique</div><div class="bar-lbl">'+hr+'</div></div>'}).join('')}
var pg=p.top_pages||[];
if(pg.length===0){document.getElementById('pages').innerHTML='<div class="empty">No page views yet</div>'}
else{document.getElementById('pages').innerHTML='<table><tr><th>Page</th><th>Views</th></tr>'+pg.map(function(x){return '<tr><td>'+esc(x.path)+'</td><td>'+fmt(x.views)+'</td></tr>'}).join('')+'</table>'}
var dl=p.top_downloads||[];
if(dl.length===0){document.getElementById('downloads').innerHTML='<div class="empty">No downloads yet</div>'}
else{document.getElementById('downloads').innerHTML='<table><tr><th>File</th><th>Count</th></tr>'+dl.map(function(x){return '<tr><td>'+esc(x.file)+'</td><td>'+fmt(x.count)+'</td></tr>'}).join('')+'</table>'}
var sb=D.by_subdomain||{};var sk=Object.keys(sb);
if(sk.length===0){document.getElementById('subs').innerHTML='<div class="empty">No subdomain data yet</div>'}
else{document.getElementById('subs').innerHTML=sk.map(function(s){return '<div class="sub-card"><div class="name">'+esc(s)+'</div><div class="stats">'+fmt(sb[s].views)+' views &middot; '+fmt(sb[s].unique)+' unique</div></div>'}).join('')}
document.getElementById('foot').textContent='Last updated: '+new Date().toLocaleTimeString()+' \u00b7 Auto-refresh in 30s'}
function fmt(n){return (n||0).toLocaleString()}
function esc(s){var d=document.createElement('div');d.textContent=s;return d.innerHTML}
render();setTimeout(function(){location.reload()},30000);
</script>
</body></html>"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_trackable_filters_health() {
        assert!(!is_trackable_request("/api/health", "Mozilla/5.0"));
        assert!(!is_trackable_request("/api/status", "Mozilla/5.0"));
        assert!(!is_trackable_request("/api/metrics", "Mozilla/5.0"));
    }

    #[test]
    fn test_is_trackable_filters_internal() {
        assert!(!is_trackable_request("/.rss/style.css", "Mozilla/5.0"));
        assert!(!is_trackable_request("/rss.js", "Mozilla/5.0"));
        assert!(!is_trackable_request("/ws/hot-reload", "Mozilla/5.0"));
        assert!(!is_trackable_request("/.well-known/acme-challenge/xxx", "Mozilla/5.0"));
    }

    #[test]
    fn test_is_trackable_filters_bots() {
        assert!(!is_trackable_request("/", "Googlebot/2.1"));
        assert!(!is_trackable_request("/", "curl/7.68.0"));
        assert!(!is_trackable_request("/", "Python-requests/2.28"));
    }

    #[test]
    fn test_is_trackable_allows_real_requests() {
        assert!(is_trackable_request("/", "Mozilla/5.0 (Macintosh)"));
        assert!(is_trackable_request("/docs", "Mozilla/5.0"));
        assert!(is_trackable_request("/about", "Safari/537.36"));
    }

    #[test]
    fn test_is_download() {
        assert!(is_download("/releases/app.zip"));
        assert!(is_download("/releases/app.tar.gz"));
        assert!(is_download("/releases/app.exe"));
        assert!(is_download("/releases/app.dmg"));
        assert!(is_download("/releases/app.AppImage"));
        assert!(!is_download("/index.html"));
        assert!(!is_download("/api/status"));
    }

    #[test]
    fn test_hash_ip_deterministic() {
        let h1 = hash_ip("192.168.1.1");
        let h2 = hash_ip("192.168.1.1");
        assert_eq!(h1, h2);
        assert_ne!(hash_ip("192.168.1.1"), hash_ip("10.0.0.1"));
    }
}
