use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub hyprland_configs: HyprlandConfigs,
    #[serde(default)]
    pub categories: HashMap<String, Category>,
    #[serde(default)]
    pub ui: UiSettings,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HyprlandConfigs {
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiSettings {
    #[serde(default = "default_show_descriptions")]
    pub show_descriptions: bool,
    #[serde(default = "default_search_threshold")]
    pub search_threshold: f64,
    #[serde(default)]
    pub show_raw_command: bool,
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    #[serde(default)]
    pub theme: ThemeSettings,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            show_descriptions: default_show_descriptions(),
            search_threshold: default_search_threshold(),
            show_raw_command: false,
            max_results: default_max_results(),
            theme: ThemeSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeSettings {
    pub name: String,
    pub colors: ThemeColors,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        Self {
            name: "catppuccin_mocha".to_string(),
            colors: ThemeColors::catppuccin_mocha(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ThemeColors {
    pub background: String,
    pub foreground: String,
    pub selected_bg: String,
    pub selected_fg: String,
    pub search_bg: String,
    pub search_fg: String,
    pub key_color: String,
    pub action_color: String,
    pub category_color: String,
    pub description_color: String,
    pub matched_color: String,
    pub border_color: String,
}

impl ThemeColors {
    pub fn catppuccin_mocha() -> Self {
        Self {
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            selected_bg: "#313244".to_string(),
            selected_fg: "#cdd6f4".to_string(),
            search_bg: "#1e1e2e".to_string(),
            search_fg: "#cdd6f4".to_string(),
            key_color: "#89b4fa".to_string(),        // Blue
            action_color: "#cdd6f4".to_string(),     // Text
            category_color: "#a6e3a1".to_string(),   // Green
            description_color: "#bac2de".to_string(), // Subtext1
            matched_color: "#f9e2af".to_string(),    // Yellow
            border_color: "#585b70".to_string(),     // Surface2
        }
    }
    
    pub fn catppuccin_latte() -> Self {
        Self {
            background: "#eff1f5".to_string(),
            foreground: "#4c4f69".to_string(),
            selected_bg: "#bcc0cc".to_string(),
            selected_fg: "#4c4f69".to_string(),
            search_bg: "#eff1f5".to_string(),
            search_fg: "#4c4f69".to_string(),
            key_color: "#1e66f5".to_string(),        // Blue
            action_color: "#4c4f69".to_string(),     // Text
            category_color: "#40a02b".to_string(),   // Green
            description_color: "#6c6f85".to_string(), // Subtext1
            matched_color: "#df8e1d".to_string(),    // Yellow
            border_color: "#9ca0b0".to_string(),     // Surface2
        }
    }
    
    pub fn catppuccin_macchiato() -> Self {
        Self {
            background: "#24273a".to_string(),
            foreground: "#cad3f5".to_string(),
            selected_bg: "#363a4f".to_string(),
            selected_fg: "#cad3f5".to_string(),
            search_bg: "#24273a".to_string(),
            search_fg: "#cad3f5".to_string(),
            key_color: "#8aadf4".to_string(),        // Blue
            action_color: "#cad3f5".to_string(),     // Text
            category_color: "#a6da95".to_string(),   // Green
            description_color: "#b8c0e0".to_string(), // Subtext1
            matched_color: "#eed49f".to_string(),    // Yellow
            border_color: "#5b6078".to_string(),     // Surface2
        }
    }
    
    pub fn catppuccin_frappe() -> Self {
        Self {
            background: "#303446".to_string(),
            foreground: "#c6d0f5".to_string(),
            selected_bg: "#414559".to_string(),
            selected_fg: "#c6d0f5".to_string(),
            search_bg: "#303446".to_string(),
            search_fg: "#c6d0f5".to_string(),
            key_color: "#8caaee".to_string(),        // Blue
            action_color: "#c6d0f5".to_string(),     // Text
            category_color: "#a6d189".to_string(),   // Green
            description_color: "#b5bfe2".to_string(), // Subtext1
            matched_color: "#e5c890".to_string(),    // Yellow
            border_color: "#626880".to_string(),     // Surface2
        }
    }
}

fn default_show_descriptions() -> bool {
    true
}

fn default_search_threshold() -> f64 {
    0.6
}

fn default_max_results() -> usize {
    50
}

impl Config {
    pub fn load(config_path: Option<PathBuf>) -> Result<Self> {
        let config_file = if let Some(path) = config_path {
            path
        } else {
            // Try default locations
            let mut default_path = dirs::config_dir()
                .context("Could not find config directory")?;
            default_path.push("hypr-showkey/showkey.yaml");
            
            if !default_path.exists() {
                // Also check current directory
                let current_dir_config = PathBuf::from("showkey.yaml");
                if current_dir_config.exists() {
                    current_dir_config
                } else {
                    return Err(anyhow::anyhow!(
                        "Configuration file not found. Please create ~/.config/hypr-showkey/showkey.yaml or provide a path with --config"
                    ));
                }
            } else {
                default_path
            }
        };

        let content = std::fs::read_to_string(&config_file)
            .with_context(|| format!("Failed to read config file: {:?}", config_file))?;
        
        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", config_file))?;
        
        Ok(config)
    }
    
    pub fn resolve_hyprland_paths(&self) -> Result<Vec<PathBuf>> {
        let mut resolved_paths = Vec::new();
        let hypr_config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("hypr");
        
        for file_path in &self.hyprland_configs.files {
            let path = if file_path.starts_with('/') {
                // Absolute path
                PathBuf::from(file_path)
            } else {
                // Relative to ~/.config/hypr/
                hypr_config_dir.join(file_path)
            };
            
            if path.exists() {
                resolved_paths.push(path);
            } else {
                eprintln!("Warning: Hyprland config file not found: {:?}", path);
            }
        }
        
        if resolved_paths.is_empty() {
            return Err(anyhow::anyhow!("No valid Hyprland config files found"));
        }
        
        Ok(resolved_paths)
    }
}
