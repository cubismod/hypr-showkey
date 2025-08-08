use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use crate::config::Config;

#[derive(Debug, Clone)]
pub struct Keybinding {
    pub key: String,
    pub action: String,
    pub description: String,
    pub category: String,
    pub raw_command: String,
}

pub struct HyprlandParser<'a> {
    config: &'a Config,
}

impl<'a> HyprlandParser<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }
    
    pub fn parse(&self) -> Result<Vec<Keybinding>> {
        let config_paths = self.config.resolve_hyprland_paths()?;
        let mut keybindings = Vec::new();
        
        for path in config_paths {
            let bindings = self.parse_file(&path)?;
            keybindings.extend(bindings);
        }
        
        Ok(keybindings)
    }
    
    fn parse_file(&self, path: &Path) -> Result<Vec<Keybinding>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {:?}", path))?;
        
        let mut keybindings = Vec::new();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments that aren't inline
            if line.is_empty() || (line.starts_with('#') && !line.contains("bind")) {
                continue;
            }
            
            if let Some(binding) = self.parse_bind_line(line) {
                keybindings.push(binding);
            }
        }
        
        Ok(keybindings)
    }
    
    fn parse_bind_line(&self, line: &str) -> Option<Keybinding> {
        // Handle comments - extract the comment part
        let (bind_part, comment) = if let Some(comment_pos) = line.find('#') {
            let bind_part = line[..comment_pos].trim();
            let comment = line[comment_pos + 1..].trim();
            (bind_part, Some(comment))
        } else {
            (line, None)
        };
        
        // Parse bind commands: bind = modifiers, key, action, params
        if !bind_part.starts_with("bind") {
            return None;
        }
        
        // Skip unbind commands (they remove keybindings)
        if bind_part.starts_with("unbind") {
            return None;
        }
        
        // Extract the part after "bind" or "binde" or "bindm"
        let after_bind = if bind_part.starts_with("binde") {
            bind_part.strip_prefix("binde")?
        } else if bind_part.starts_with("bindm") {
            bind_part.strip_prefix("bindm")?
        } else {
            bind_part.strip_prefix("bind")?
        };
        
        // Handle the = sign properly
        let bind_content = after_bind.trim();
        let bind_content = if bind_content.starts_with('=') {
            bind_content[1..].trim()
        } else {
            bind_content
        };
        
        // The format is: modifiers, key, action, [params...]
        // Split by commas, but be careful with nested commas in exec commands
        let parts = self.split_bind_parts(bind_content);
        if parts.len() < 3 {
            return None;
        }
        
        // Parse the parts correctly
        let modifiers = parts[0].trim();
        let key = parts[1].trim();
        let action = parts[2].trim();
        let params = if parts.len() > 3 {
            parts[3..].join(",").trim().to_string()
        } else {
            String::new()
        };
        
        // Combine modifiers and key
        let modifiers_and_key = if modifiers.is_empty() {
            key.to_string()
        } else {
            format!("{} {}", modifiers, key)
        };
        
        // Filter out empty or unbound keybindings
        if action.is_empty() {
            return None;
        }
        
        // Format the key combination  
        let formatted_key = self.format_key_combination(&modifiers_and_key);
        
        // Create description from comment or action
        let description = if let Some(comment) = comment {
            comment.to_string()
        } else {
            self.generate_description(action, &params)
        };
        
        // Determine category
        let category = self.determine_category(action, &params, &description);
        
        // Create the full action string
        let full_action = if params.is_empty() {
            action.to_string()
        } else {
            format!("{}, {}", action, params)
        };
        
        Some(Keybinding {
            key: formatted_key,
            action: full_action.clone(),
            description,
            category,
            raw_command: format!("bind = {}", bind_content),
        })
    }
    
    fn split_bind_parts(&self, content: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut paren_depth = 0;
        let mut chars = content.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    in_quotes = !in_quotes;
                    current.push(ch);
                }
                '(' => {
                    paren_depth += 1;
                    current.push(ch);
                }
                ')' => {
                    paren_depth -= 1;
                    current.push(ch);
                }
                ',' if !in_quotes && paren_depth == 0 => {
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                    }
                    current = String::new();
                }
                _ => {
                    current.push(ch);
                }
            }
        }
        
        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }
        
        parts
    }
    
    fn format_key_combination(&self, modifiers_and_key: &str) -> String {
        // Replace variables if they exist
        let formatted = modifiers_and_key
            .replace("$mainMod", "Super")
            .replace("$shiftMod", "Shift");
        
        // Split by whitespace and handle the key combination
        let parts: Vec<&str> = formatted.split_whitespace().collect();
        
        if parts.is_empty() {
            return "Unknown".to_string();
        }
        
        // Last part is the key, everything before are modifiers
        if parts.len() == 1 {
            parts[0].to_string()
        } else {
            let modifiers = &parts[..parts.len() - 1];
            let key = parts[parts.len() - 1];
            
            // Join modifiers with + and add key at the end
            let modifier_string = modifiers.join(" + ");
            format!("{} + {}", modifier_string, key)
        }
    }
    
    fn generate_description(&self, action: &str, params: &str) -> String {
        match action {
            "exec" => {
                if params.contains("terminal") {
                    "Open terminal".to_string()
                } else if params.contains("browser") {
                    "Open browser".to_string()
                } else if params.contains("filemanager") {
                    "Open file manager".to_string()
                } else {
                    format!("Execute: {}", params)
                }
            }
            "killactive" => "Kill active window".to_string(),
            "fullscreen" => {
                if params == "0" {
                    "Toggle fullscreen".to_string()
                } else {
                    "Maximize window".to_string()
                }
            }
            "togglefloating" => "Toggle floating mode".to_string(),
            "workspace" => format!("Switch to workspace {}", params),
            "movetoworkspace" => format!("Move window to workspace {}", params),
            _ => format!("{} {}", action, params).trim().to_string(),
        }
    }
    
    fn determine_category(&self, action: &str, params: &str, description: &str) -> String {
        let search_text = format!("{} {} {}", action, params, description).to_lowercase();
        
        for (_category_id, category) in &self.config.categories {
            for keyword in &category.keywords {
                if search_text.contains(&keyword.to_lowercase()) {
                    return category.name.clone();
                }
            }
        }
        
        "Other".to_string()
    }
}