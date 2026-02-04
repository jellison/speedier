use crate::calc::Entry;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Preferences {
    pub window_width: Option<f32>,
    pub window_height: Option<f32>,
    pub reference_visible: Option<bool>,
    #[serde(default)]
    pub last_result: Option<f64>,
    #[serde(default)]
    pub history: Vec<Entry>,
}

impl Preferences {
    pub fn load() -> Option<Self> {
        let path = preferences_path()?;
        let data = fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = preferences_path().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "config directory not found")
        })?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string_pretty(self).unwrap_or_default();
        fs::write(path, data)
    }

    pub fn window_size(&self) -> Option<(f32, f32)> {
        match (self.window_width, self.window_height) {
            (Some(w), Some(h)) => Some((w, h)),
            _ => None,
        }
    }
}

pub fn preferences_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "speedier", "Speedier")
        .map(|proj| proj.config_dir().join("settings.json"))
}
