use crate::core::prelude::*;
use std::collections::HashMap;

pub mod command;
pub use command::ThemeCommand;

#[derive(Debug, Clone)]
pub struct ThemeDefinition {
    pub input_text: String,
    pub input_bg: String,
    pub output_text: String,
    pub output_bg: String,
    pub input_cursor_prefix: String,
    pub input_cursor_color: String,
    pub input_cursor: String,
    pub output_cursor: String,
    pub output_cursor_color: String,
}

#[derive(Debug)]
pub struct ThemeSystem {
    themes: HashMap<String, ThemeDefinition>,
    current_name: String,
    config_paths: Vec<std::path::PathBuf>,
}

impl ThemeSystem {
    pub fn load() -> Result<Self> {
        let config_paths = crate::setup::setup_toml::get_config_paths();
        let themes = Self::load_themes_from_paths(&config_paths)?;
        let current_name = Self::load_current_theme_name(&config_paths).unwrap_or_else(|| {
            themes
                .keys()
                .next()
                .cloned()
                .unwrap_or_else(|| "default".to_string())
        });

        log::info!(
            "{} themes loaded: {}",
            themes.len(),
            themes.keys().cloned().collect::<Vec<_>>().join(", ")
        );

        Ok(Self {
            themes,
            current_name,
            config_paths,
        })
    }

    pub fn show_status(&self) -> String {
        if self.themes.is_empty() {
            return "No themes available! Add [theme.xyz] sections to rush.toml.".to_string();
        }
        format!(
            "Current theme: {} (from TOML)\nAvailable: {}",
            self.current_name.to_uppercase(),
            self.themes.keys().cloned().collect::<Vec<_>>().join(", ")
        )
    }

    pub fn change_theme(&mut self, theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        if !self.themes.contains_key(&theme_name_lower) {
            return Ok(if self.themes.is_empty() {
                "No themes available! Add [theme.xyz] sections to rush.toml.".to_string()
            } else {
                format!(
                    "Theme '{}' not found. Available: {}",
                    theme_name,
                    self.themes.keys().cloned().collect::<Vec<_>>().join(", ")
                )
            });
        }

        self.current_name = theme_name_lower.clone();

        // Log cursor details
        if let Some(theme_def) = self.themes.get(&theme_name_lower) {
            log::info!(
                "Theme '{}': input_cursor='{}' ({}), output_cursor='{}' ({}), prefix='{}'",
                theme_name_lower.to_uppercase(),
                theme_def.input_cursor,
                theme_def.input_cursor_color,
                theme_def.output_cursor,
                theme_def.output_cursor_color,
                theme_def.input_cursor_prefix
            );
        }

        // Async save
        let name_clone = theme_name_lower.clone();
        let paths_clone = self.config_paths.clone();
        tokio::spawn(async move {
            if let Err(e) = Self::save_current_theme_to_config(&paths_clone, &name_clone).await {
                log::error!("Failed to save theme: {}", e);
            }
        });

        Ok(format!(
            "__LIVE_THEME_UPDATE__{}__MESSAGE__Theme changed to: {}",
            theme_name_lower,
            theme_name_lower.to_uppercase()
        ))
    }

    pub fn preview_theme(&self, theme_name: &str) -> Result<String> {
        let theme_name_lower = theme_name.to_lowercase();

        if let Some(theme_def) = self.themes.get(&theme_name_lower) {
            Ok(format!("Theme '{}' Preview:\nInput: {} on {}\nOutput: {} on {}\nCursor Prefix: '{}' in {}\nInput Cursor: {}\nOutput Cursor: {} in {}\n\nSource: [theme.{}] in rush.toml",
                theme_name_lower.to_uppercase(), theme_def.input_text, theme_def.input_bg,
                theme_def.output_text, theme_def.output_bg, theme_def.input_cursor_prefix,
                theme_def.input_cursor_color, theme_def.input_cursor, theme_def.output_cursor,
                theme_def.output_cursor_color, theme_name_lower))
        } else {
            Ok(format!(
                "Theme '{}' not found. Available: {}",
                theme_name,
                self.themes.keys().cloned().collect::<Vec<_>>().join(", ")
            ))
        }
    }

    pub fn debug_theme_details(&self, theme_name: &str) -> String {
        if let Some(theme_def) = self.themes.get(&theme_name.to_lowercase()) {
            format!("Theme '{}':\ninput_text: '{}'\ninput_bg: '{}'\noutput_text: '{}'\noutput_bg: '{}'\ninput_cursor_prefix: '{}'\ninput_cursor_color: '{}'\ninput_cursor: '{}'\noutput_cursor: '{}'\noutput_cursor_color: '{}'",
                theme_name.to_uppercase(), theme_def.input_text, theme_def.input_bg,
                theme_def.output_text, theme_def.output_bg, theme_def.input_cursor_prefix,
                theme_def.input_cursor_color, theme_def.input_cursor, theme_def.output_cursor,
                theme_def.output_cursor_color)
        } else {
            format!("Theme '{}' not found!", theme_name)
        }
    }

    pub fn theme_exists(&self, theme_name: &str) -> bool {
        self.themes.contains_key(&theme_name.to_lowercase())
    }
    pub fn get_theme(&self, theme_name: &str) -> Option<&ThemeDefinition> {
        self.themes.get(&theme_name.to_lowercase())
    }
    pub fn get_available_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.themes.keys().cloned().collect();
        names.sort();
        names
    }
    pub fn get_current_name(&self) -> &str {
        &self.current_name
    }

    fn load_themes_from_paths(
        config_paths: &[std::path::PathBuf],
    ) -> Result<HashMap<String, ThemeDefinition>> {
        for path in config_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(themes) = Self::parse_themes_from_toml(&content) {
                        return Ok(themes);
                    }
                }
            }
        }
        Ok(HashMap::new())
    }

    fn parse_themes_from_toml(content: &str) -> Result<HashMap<String, ThemeDefinition>> {
        let mut themes = HashMap::new();
        let mut current_theme: Option<String> = None;
        let mut current_data = HashMap::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some(theme_name) = trimmed
                .strip_prefix("[theme.")
                .and_then(|s| s.strip_suffix(']'))
            {
                Self::finalize_theme(&mut themes, current_theme.take(), &mut current_data);
                current_theme = Some(theme_name.to_lowercase());
            } else if trimmed.starts_with('[') && !trimmed.starts_with("[theme.") {
                Self::finalize_theme(&mut themes, current_theme.take(), &mut current_data);
            } else if current_theme.is_some() && trimmed.contains('=') {
                if let Some((key, value)) = trimmed.split_once('=') {
                    let clean_value = value.trim().trim_matches('"').trim_matches('\'');
                    if !clean_value.is_empty() {
                        current_data.insert(key.trim().to_string(), clean_value.to_string());
                    }
                }
            }
        }
        Self::finalize_theme(&mut themes, current_theme, &mut current_data);
        Ok(themes)
    }

    fn finalize_theme(
        themes: &mut HashMap<String, ThemeDefinition>,
        theme_name: Option<String>,
        data: &mut HashMap<String, String>,
    ) {
        if let Some(name) = theme_name {
            if let Some(theme_def) = Self::build_theme_from_data(data) {
                themes.insert(name, theme_def);
            }
            data.clear();
        }
    }

    fn build_theme_from_data(data: &HashMap<String, String>) -> Option<ThemeDefinition> {
        Some(ThemeDefinition {
            input_text: data.get("input_text")?.clone(),
            input_bg: data.get("input_bg")?.clone(),
            output_text: data.get("output_text")?.clone(),
            output_bg: data.get("output_bg")?.clone(),
            input_cursor_prefix: data
                .get("input_cursor_prefix")
                .or(data.get("prompt_text"))
                .unwrap_or(&"/// ".to_string())
                .clone(),
            input_cursor_color: data
                .get("input_cursor_color")
                .or(data.get("prompt_color"))
                .unwrap_or(&"LightBlue".to_string())
                .clone(),
            input_cursor: data
                .get("input_cursor")
                .or(data.get("prompt_cursor"))
                .unwrap_or(&"PIPE".to_string())
                .clone(),
            output_cursor: data
                .get("output_cursor")
                .unwrap_or(&"PIPE".to_string())
                .clone(),
            output_cursor_color: data
                .get("output_cursor_color")
                .or(data.get("output_color"))
                .unwrap_or(&"White".to_string())
                .clone(),
        })
    }

    fn load_current_theme_name(config_paths: &[std::path::PathBuf]) -> Option<String> {
        for path in config_paths {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Some(theme) = Self::extract_current_theme_from_toml(&content) {
                        return Some(theme);
                    }
                }
            }
        }
        None
    }

    fn extract_current_theme_from_toml(content: &str) -> Option<String> {
        let mut in_general = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "[general]" {
                in_general = true;
            } else if trimmed.starts_with('[') {
                in_general = false;
            } else if in_general && trimmed.starts_with("current_theme") {
                return trimmed
                    .split('=')
                    .nth(1)?
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string()
                    .into();
            }
        }
        None
    }

    async fn save_current_theme_to_config(
        config_paths: &[std::path::PathBuf],
        theme_name: &str,
    ) -> Result<()> {
        for path in config_paths {
            if path.exists() {
                let content = tokio::fs::read_to_string(path)
                    .await
                    .map_err(AppError::Io)?;
                let updated = Self::update_current_theme_in_toml(&content, theme_name)?;
                tokio::fs::write(path, updated)
                    .await
                    .map_err(AppError::Io)?;
                return Ok(());
            }
        }
        Err(AppError::Validation("No config file found".to_string()))
    }

    fn update_current_theme_in_toml(content: &str, theme_name: &str) -> Result<String> {
        let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
        let mut in_general = false;
        let mut updated = false;

        for line in lines.iter_mut() {
            let trimmed = line.trim();
            if trimmed == "[general]" {
                in_general = true;
            } else if trimmed.starts_with('[') {
                in_general = false;
            } else if in_general && trimmed.starts_with("current_theme") {
                *line = format!("current_theme = \"{}\"", theme_name);
                updated = true;
            }
        }

        if !updated {
            if let Some(general_idx) = lines.iter().position(|line| line.trim() == "[general]") {
                let insert_idx = lines
                    .iter()
                    .enumerate()
                    .skip(general_idx + 1)
                    .find(|(_, line)| line.trim().starts_with('['))
                    .map(|(idx, _)| idx)
                    .unwrap_or(lines.len());
                lines.insert(insert_idx, format!("current_theme = \"{}\"", theme_name));
            }
        }
        Ok(lines.join("\n"))
    }
}
