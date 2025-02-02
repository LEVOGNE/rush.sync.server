use crate::constants::{CONFIG_PATHS, DEFAULT_BUFFER_SIZE, DEFAULT_POLL_RATE};
use crate::prelude::*;

// Interne Struktur für Serialisierung/Deserialisierung
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    general: GeneralConfig,
    theme: ThemeConfig,
    prompt: PromptConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct GeneralConfig {
    max_messages: usize,
    typewriter_delay: u64,
    input_max_length: usize,
    max_history: usize,
    poll_rate: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ThemeConfig {
    input_text: String,
    cursor: String,
    output_text: String,
    border: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PromptConfig {
    text: String,
    color: String,
}

// Öffentliche Strukturen
pub struct Config {
    config_path: Option<String>,
    pub max_messages: usize,
    pub typewriter_delay: Duration,
    pub input_max_length: usize,
    pub max_history: usize,
    pub poll_rate: Duration,
    pub theme: Theme,
    pub prompt: Prompt,
    pub debug_info: Option<String>,
}

pub struct Theme {
    pub input_text: AppColor,
    pub cursor: AppColor,
    pub output_text: AppColor,
    pub border: AppColor,
}

pub struct Prompt {
    pub text: String,
    pub color: AppColor,
}

impl Config {
    pub async fn load() -> Result<Self> {
        let mut last_error = None;

        for &path in CONFIG_PATHS.iter() {
            match Self::from_file(path).await {
                Ok(mut config) => {
                    config.debug_info = Some(format!(
                        "Konfiguration geladen aus '{}': Prompt='{}', Color={:?}",
                        path, config.prompt.text, config.prompt.color
                    ));
                    return Ok(config);
                }
                Err(e) => {
                    last_error = Some(e);
                    continue;
                }
            }
        }

        log::warn!("Keine Konfigurationsdatei gefunden, verwende Defaults");
        if let Some(err) = last_error {
            log::debug!("Letzter Fehler beim Laden: {:?}", err);
        }

        let mut default_config = Self::default();
        default_config.debug_info = Some(format!(
            "Keine Konfigurationsdatei gefunden in {:?}, verwende Defaults",
            *CONFIG_PATHS
        ));
        Ok(default_config)
    }

    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| AppError::Io(e))?;

        let config_file: ConfigFile = toml::from_str(&content)
            .map_err(|e| AppError::Validation(format!("Ungültiges TOML-Format: {}", e)))?;

        log::debug!(
            "Theme-Konfiguration geladen: input_text={}, cursor={}, output_text={}, border={}",
            config_file.theme.input_text,
            config_file.theme.cursor,
            config_file.theme.output_text,
            config_file.theme.border
        );

        Ok(Self {
            config_path: Some(path.as_ref().to_string_lossy().into_owned()),
            max_messages: config_file.general.max_messages,
            typewriter_delay: Duration::from_millis(config_file.general.typewriter_delay),
            input_max_length: config_file.general.input_max_length,
            max_history: config_file.general.max_history,
            poll_rate: Duration::from_millis(config_file.general.poll_rate),
            theme: Theme::from_config(&config_file.theme)?,
            prompt: Prompt::from_config(&config_file.prompt)?,
            debug_info: None,
        })
    }

    pub async fn save(&self) -> Result<()> {
        if let Some(path) = &self.config_path {
            let config_file = ConfigFile {
                general: GeneralConfig {
                    max_messages: self.max_messages,
                    typewriter_delay: self.typewriter_delay.as_millis() as u64,
                    input_max_length: self.input_max_length,
                    max_history: self.max_history,
                    poll_rate: self.poll_rate.as_millis() as u64,
                },
                theme: ThemeConfig {
                    input_text: self.theme.input_text.to_string(),
                    cursor: self.theme.cursor.to_string(),
                    output_text: self.theme.output_text.to_string(),
                    border: self.theme.border.to_string(),
                },
                prompt: PromptConfig {
                    text: self.prompt.text.clone(),
                    color: self.prompt.color.to_string(),
                },
            };

            let content = toml::to_string_pretty(&config_file)
                .map_err(|e| AppError::Validation(format!("Serialisierungsfehler: {}", e)))?;

            tokio::fs::write(path, content)
                .await
                .map_err(|e| AppError::Io(e))?;
        }
        Ok(())
    }
}

impl Theme {
    fn from_config(config: &ThemeConfig) -> Result<Self> {
        Ok(Self {
            input_text: AppColor::from_string(&config.input_text)?,
            cursor: AppColor::from_string(&config.cursor)?,
            output_text: AppColor::from_string(&config.output_text)?,
            border: AppColor::from_string(&config.border)?,
        })
    }
}

impl Prompt {
    fn from_config(config: &PromptConfig) -> Result<Self> {
        Ok(Self {
            text: config.text.clone(),
            color: AppColor::from_string(&config.color)?,
        })
    }
}

// Default-Implementierungen bleiben unverändert
impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: None,
            max_messages: DEFAULT_BUFFER_SIZE,
            typewriter_delay: Duration::from_millis(50),
            input_max_length: DEFAULT_BUFFER_SIZE,
            max_history: 30,
            poll_rate: Duration::from_millis(DEFAULT_POLL_RATE),
            theme: Theme::default(),
            prompt: Prompt::default(),
            debug_info: None,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            input_text: AppColor::new(Color::Yellow),
            cursor: AppColor::new(Color::Yellow),
            output_text: AppColor::new(Color::Green),
            border: AppColor::new(Color::DarkGray),
        }
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            text: "/// ".to_string(),
            color: AppColor::new(Color::Yellow),
        }
    }
}
