use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(rename = "BaiduAppId", default)]
    pub baidu_app_id: String,
    #[serde(rename = "BaiduSecret", default)]
    pub baidu_secret: String,
    #[serde(rename = "HoldMilliseconds", default = "default_hold_ms")]
    pub hold_ms: u64,
}

fn default_hold_ms() -> u64 {
    500
}

impl Config {
    pub fn path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
        PathBuf::from(appdata).join("FloatTrans").join("config.json")
    }

    pub fn load() -> Config {
        let p = Self::path();
        match fs::read_to_string(&p) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| {
                let c = Config::default();
                c.save();
                c
            }),
            Err(_) => {
                let c = Config::default();
                c.save();
                c
            }
        }
    }

    pub fn save(&self) {
        let p = Self::path();
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = fs::write(&p, s);
        }
    }
}
