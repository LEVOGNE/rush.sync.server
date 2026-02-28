use crate::core::helpers::get_base_dir;
use crate::core::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const DEFAULT_REMOTE_PORT: u16 = 22;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteProfile {
    pub host: String,
    pub user: String,
    #[serde(default = "default_remote_port")]
    pub port: u16,
    pub remote_path: String,
    #[serde(default)]
    pub identity_file: Option<String>,
}

fn default_remote_port() -> u16 {
    DEFAULT_REMOTE_PORT
}

impl RemoteProfile {
    pub fn new(
        user: String,
        host: String,
        remote_path: String,
        port: u16,
        identity_file: Option<String>,
    ) -> Result<Self> {
        if user.trim().is_empty() {
            return Err(AppError::Validation(
                "Remote user cannot be empty".to_string(),
            ));
        }
        if host.trim().is_empty() {
            return Err(AppError::Validation(
                "Remote host cannot be empty".to_string(),
            ));
        }
        if remote_path.trim().is_empty() {
            return Err(AppError::Validation(
                "Remote path cannot be empty".to_string(),
            ));
        }
        if remote_path.contains('\n') || remote_path.contains('\r') {
            return Err(AppError::Validation(
                "Remote path contains invalid newline characters".to_string(),
            ));
        }
        if !remote_path.starts_with('/') {
            return Err(AppError::Validation(
                "Remote path must be absolute (start with '/')".to_string(),
            ));
        }
        if remote_path.contains("..") {
            return Err(AppError::Validation(
                "Remote path must not contain '..'".to_string(),
            ));
        }
        if port == 0 {
            return Err(AppError::Validation("Remote port must be > 0".to_string()));
        }

        if let Some(ref identity) = identity_file {
            if identity.contains("..") {
                return Err(AppError::Validation(
                    "Identity file path must not contain '..'".to_string(),
                ));
            }
        }

        Ok(Self {
            host,
            user,
            port,
            remote_path,
            identity_file,
        })
    }

    pub fn ssh_target(&self) -> String {
        format!("{}@{}", self.user, self.host)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProfilesFile {
    #[serde(default)]
    profiles: HashMap<String, RemoteProfile>,
}

#[derive(Debug, Clone)]
pub struct RemoteProfileStore {
    path: PathBuf,
}

impl RemoteProfileStore {
    pub fn new() -> Result<Self> {
        let base_dir = get_base_dir()?;
        Ok(Self {
            path: base_dir.join(".rss").join("remotes.toml"),
        })
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn list(&self) -> Result<Vec<(String, RemoteProfile)>> {
        let file = self.load_file()?;
        let mut entries: Vec<_> = file.profiles.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(entries)
    }

    pub fn get(&self, name: &str) -> Result<RemoteProfile> {
        let file = self.load_file()?;
        file.profiles.get(name).cloned().ok_or_else(|| {
            AppError::Validation(format!(
                "Remote profile '{}' not found. Use 'remote list' to inspect configured remotes.",
                name
            ))
        })
    }

    pub fn exists(&self, name: &str) -> Result<bool> {
        let file = self.load_file()?;
        Ok(file.profiles.contains_key(name))
    }

    pub fn upsert(&self, name: &str, profile: RemoteProfile) -> Result<()> {
        validate_profile_name(name)?;

        let mut file = self.load_file()?;
        file.profiles.insert(name.to_string(), profile);
        self.save_file(&file)
    }

    pub fn remove(&self, name: &str) -> Result<()> {
        let mut file = self.load_file()?;
        if file.profiles.remove(name).is_none() {
            return Err(AppError::Validation(format!(
                "Remote profile '{}' not found",
                name
            )));
        }
        self.save_file(&file)
    }

    fn load_file(&self) -> Result<ProfilesFile> {
        if !self.path.exists() {
            return Ok(ProfilesFile::default());
        }

        let content = std::fs::read_to_string(&self.path).map_err(AppError::Io)?;
        toml::from_str::<ProfilesFile>(&content)
            .map_err(|e| AppError::Validation(format!("Failed to parse remotes file: {}", e)))
    }

    fn save_file(&self, file: &ProfilesFile) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }

        let serialized = toml::to_string_pretty(file).map_err(|e| {
            AppError::Validation(format!("Failed to serialize remotes file: {}", e))
        })?;

        std::fs::write(&self.path, serialized).map_err(AppError::Io)
    }
}

pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation(
            "Remote profile name cannot be empty".to_string(),
        ));
    }
    if name.len() > 64 {
        return Err(AppError::Validation(
            "Remote profile name too long (max 64 chars)".to_string(),
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::Validation(
            "Remote profile name may only contain a-z, A-Z, 0-9, '-' and '_'".to_string(),
        ));
    }
    Ok(())
}

pub fn parse_user_host(input: &str) -> Result<(String, String)> {
    let (user, host) = input
        .split_once('@')
        .ok_or_else(|| AppError::Validation("Expected '<user>@<host>' format".to_string()))?;

    let user = user.trim();
    let host = host.trim();

    if user.is_empty() || host.is_empty() {
        return Err(AppError::Validation(
            "Both user and host are required in '<user>@<host>'".to_string(),
        ));
    }

    if user.contains(|c: char| c.is_whitespace() || c == ';' || c == '&' || c == '|' || c == '$')
    {
        return Err(AppError::Validation(
            "User contains invalid characters".to_string(),
        ));
    }
    if host.contains(|c: char| c.is_whitespace() || c == ';' || c == '&' || c == '|' || c == '$')
    {
        return Err(AppError::Validation(
            "Host contains invalid characters".to_string(),
        ));
    }

    Ok((user.to_string(), host.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_user_host_ok() {
        let (user, host) = parse_user_host("deploy@example.com").expect("must parse");
        assert_eq!(user, "deploy");
        assert_eq!(host, "example.com");
    }

    #[test]
    fn validate_profile_name_rejects_space() {
        let res = validate_profile_name("prod west");
        assert!(res.is_err());
    }

    #[test]
    fn rejects_relative_remote_path() {
        let res = RemoteProfile::new("u".into(), "h".into(), "relative/path".into(), 22, None);
        assert!(res.is_err());
    }

    #[test]
    fn rejects_path_traversal_remote_path() {
        let res = RemoteProfile::new("u".into(), "h".into(), "/opt/../etc".into(), 22, None);
        assert!(res.is_err());
    }

    #[test]
    fn rejects_identity_with_traversal() {
        let res = RemoteProfile::new(
            "u".into(),
            "h".into(),
            "/opt/app".into(),
            22,
            Some("../../etc/passwd".into()),
        );
        assert!(res.is_err());
    }

    #[test]
    fn accepts_valid_profile() {
        let res = RemoteProfile::new(
            "deploy".into(),
            "example.com".into(),
            "/opt/app".into(),
            22,
            Some("~/.ssh/id_ed25519".into()),
        );
        assert!(res.is_ok());
    }

    #[test]
    fn parse_user_host_rejects_shell_chars() {
        assert!(parse_user_host("user;cmd@host").is_err());
        assert!(parse_user_host("user@host$(cmd)").is_err());
    }

    #[test]
    fn store_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!(
            "rush-sync-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let file_path = temp_dir.join("remotes.toml");
        let store = RemoteProfileStore::with_path(file_path.clone());

        let profile = RemoteProfile::new(
            "deploy".to_string(),
            "example.com".to_string(),
            "/opt/app".to_string(),
            22,
            None,
        )
        .expect("profile");

        store.upsert("prod", profile).expect("save");
        let loaded = store.get("prod").expect("get");
        assert_eq!(loaded.user, "deploy");
        assert_eq!(loaded.host, "example.com");

        let _ = std::fs::remove_file(file_path);
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
