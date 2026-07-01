use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SortRule {
    pub id: String,
    pub name: String,
    pub condition: RuleCondition,
    pub target_fence_id: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RuleCondition {
    Extension(String),
    NameContains(String),
    NamePrefix(String),
}

fn config_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config")
}

fn rules_file() -> PathBuf {
    config_dir().join("rules.json")
}

#[tauri::command]
pub fn save_rules(rules: Vec<SortRule>) -> Result<(), String> {
    let path = rules_file();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir: {}", e))?;
    }
    let json =
        serde_json::to_string_pretty(&rules).map_err(|e| format!("Serialize error: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Write error: {}", e))
}

#[tauri::command]
pub fn load_rules() -> Result<Vec<SortRule>, String> {
    let path = rules_file();
    if !path.exists() {
        return Ok(vec![]);
    }
    let json = fs::read_to_string(&path).map_err(|e| format!("Read error: {}", e))?;
    serde_json::from_str(&json).map_err(|e| format!("Parse error: {}", e))
}

#[tauri::command]
pub fn match_rules(filename: String, rules: Vec<SortRule>) -> Option<String> {
    let lower = filename.to_lowercase();
    for rule in &rules {
        if !rule.enabled {
            continue;
        }
        let matched = match &rule.condition {
            RuleCondition::Extension(ext) => {
                let dot_ext = format!(".{}", ext.to_lowercase());
                lower.ends_with(&dot_ext)
            }
            RuleCondition::NameContains(sub) => lower.contains(&sub.to_lowercase()),
            RuleCondition::NamePrefix(pfx) => lower.starts_with(&pfx.to_lowercase()),
        };
        if matched {
            return Some(rule.target_fence_id.clone());
        }
    }
    None
}
