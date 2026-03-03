use base64::Engine;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    #[serde(default)]
    pub custom_404_enabled: bool,
    #[serde(default = "default_404_path")]
    pub custom_404_path: String,
    #[serde(default)]
    pub pin_enabled: bool,
    #[serde(default)]
    pub pin_code: String,
}

fn default_404_path() -> String {
    "404.html".to_string()
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            custom_404_enabled: false,
            custom_404_path: default_404_path(),
            pin_enabled: false,
            pin_code: String::new(),
        }
    }
}

impl ServerSettings {
    /// Get the settings file path for a server directory
    pub fn settings_path(server_dir: &Path) -> PathBuf {
        server_dir.join(".rss-settings.json")
    }

    /// Load settings from the server directory
    pub fn load(server_dir: &Path) -> Self {
        let path = Self::settings_path(server_dir);
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(e) => {
                    log::warn!("Failed to read settings: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// Save settings to the server directory
    pub fn save(&self, server_dir: &Path) -> Result<(), std::io::Error> {
        let path = Self::settings_path(server_dir);
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(&path, content)
    }

    /// Get the server directory path from server name and port
    pub fn get_server_dir(server_name: &str, port: u16) -> Option<PathBuf> {
        let base_dir = crate::core::helpers::get_base_dir().ok()?;
        Some(
            base_dir
                .join("www")
                .join(format!("{}-[{}]", server_name, port)),
        )
    }

    /// Encode a PIN for storage (Base64 obfuscation)
    pub fn encode_pin(plain: &str) -> String {
        base64::engine::general_purpose::STANDARD.encode(plain.as_bytes())
    }

    /// Decode a stored PIN
    fn decode_pin(encoded: &str) -> Option<String> {
        base64::engine::general_purpose::STANDARD
            .decode(encoded)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    /// Verify a plain PIN against the stored encoded PIN
    pub fn verify_pin(&self, input: &str) -> bool {
        if !self.pin_enabled || self.pin_code.is_empty() {
            return true;
        }
        // Try decoding (new format) or direct compare (legacy)
        match Self::decode_pin(&self.pin_code) {
            Some(decoded) => decoded == input,
            None => self.pin_code == input,
        }
    }

    /// Auto-create the 404.html file if it doesn't exist
    pub fn ensure_404_page(&self, server_dir: &Path, server_name: &str) {
        if !self.custom_404_enabled {
            return;
        }
        let page_path = server_dir.join(&self.custom_404_path);
        if page_path.exists() {
            return;
        }
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>404 — {name}</title>
<link rel="icon" href="/.rss/favicon.svg" type="image/svg+xml">
<style>
:root {{
  --bg: #0a0a0f;
  --card: #12121a;
  --border: #2a2a3a;
  --text: #e4e4ef;
  --dim: #8888a0;
  --muted: #55556a;
  --accent: #6c63ff;
}}
*{{ margin:0; padding:0; box-sizing:border-box; }}
body {{
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
  background: var(--bg);
  color: var(--text);
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}}
.grain {{
  position:fixed; top:0; left:0; width:100%; height:100%;
  background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)' opacity='0.03'/%3E%3C/svg%3E");
  pointer-events:none; z-index:1000;
}}
.orb {{
  position:fixed; border-radius:50%; filter:blur(90px); opacity:0.08; pointer-events:none;
}}
.orb.p {{ background:#6c63ff; width:350px; height:350px; top:-80px; right:-50px; }}
.orb.c {{ background:#22d3ee; width:250px; height:250px; bottom:100px; left:-60px; }}
.box {{
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 48px;
  text-align: center;
  max-width: 440px;
  position: relative;
  z-index: 1;
}}
.code {{
  font-size: 72px;
  font-weight: 800;
  color: var(--accent);
  letter-spacing: -2px;
  line-height: 1;
  margin-bottom: 12px;
}}
.msg {{
  font-size: 16px;
  color: var(--dim);
  margin-bottom: 28px;
  line-height: 1.5;
}}
.server {{
  font-family: 'SF Mono', 'Fira Code', monospace;
  font-size: 11px;
  font-weight: 700;
  color: #fff;
  background: var(--accent);
  padding: 3px 12px;
  border-radius: 100px;
  display: inline-block;
  margin-bottom: 24px;
}}
.links {{
  display: flex;
  gap: 8px;
  justify-content: center;
  flex-wrap: wrap;
}}
.links a {{
  display: inline-block;
  padding: 8px 20px;
  border: 1px solid var(--border);
  border-radius: 100px;
  color: var(--dim);
  text-decoration: none;
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  transition: all 0.15s;
}}
.links a:hover {{
  border-color: var(--accent);
  color: var(--accent);
}}
.hint {{
  font-size: 11px;
  color: var(--muted);
  margin-top: 20px;
}}
</style>
</head>
<body>
<div class="grain"></div>
<div class="orb p"></div>
<div class="orb c"></div>
<div class="box">
  <span class="server">{name}</span>
  <div class="code">404</div>
  <div class="msg">This page doesn't exist yet.</div>
  <div class="links">
    <a href="/">Home</a>
    <a href="/.rss/">Dashboard</a>
  </div>
  <div class="hint">Edit this file: {path}</div>
</div>
</body>
</html>"#,
            name = server_name,
            path = self.custom_404_path
        );
        if let Err(e) = std::fs::write(&page_path, html) {
            log::error!("Failed to create 404 page: {}", e);
        } else {
            log::info!("Created custom 404 page: {:?}", page_path);
        }
    }
}
