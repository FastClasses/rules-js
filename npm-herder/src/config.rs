use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

pub static DEFAULT_CONFIG: &'static str = include_str!("../default_herder.toml");

#[derive(Debug, Deserialize)]
pub struct HerderConfig {
    #[serde(default = "default_vendor_dir")]
    pub vendor_dir: String,

    #[serde(default = "default_lockfile")]
    pub lockfile: String,

    #[serde(default)]
    pub production: bool,

    #[serde(default)]
    pub buck: BuckConfig,

    #[serde(default)]
    pub vendor: VendorConfig,

    #[serde(default)]
    pub platform: HashMap<String, PlatformConfig>,
}

#[derive(Debug, Deserialize)]
pub struct BuckConfig {
    #[serde(default = "default_rules_path")]
    pub rules_path: String,

    #[serde(default = "default_buck_file_name")]
    pub file_name: String,

    #[serde(default)]
    pub generated_file_header: String,
}

#[derive(Debug, Deserialize)]
pub struct VendorConfig {
    #[serde(default = "default_parallel")]
    pub parallel: usize,

    #[serde(default = "default_true")]
    pub clean_stale: bool,
}

#[derive(Debug, Deserialize)]
pub struct PlatformConfig {
    pub os: String,
    pub arch: String,
}

fn default_vendor_dir() -> String {
    "vendor".to_string()
}
fn default_lockfile() -> String {
    "pnpm-lock.yaml".to_string()
}
fn default_rules_path() -> String {
    "//rules/js:js_library.bzl".to_string()
}
fn default_buck_file_name() -> String {
    "BUCK".to_string()
}
fn default_parallel() -> usize {
    8
}
fn default_true() -> bool {
    true
}

impl Default for BuckConfig {
    fn default() -> Self {
        Self {
            rules_path: default_rules_path(),
            file_name: default_buck_file_name(),
            generated_file_header: String::new(),
        }
    }
}

impl Default for VendorConfig {
    fn default() -> Self {
        Self {
            parallel: default_parallel(),
            clean_stale: true,
        }
    }
}

impl Default for HerderConfig {
    fn default() -> Self {
        Self {
            vendor_dir: default_vendor_dir(),
            lockfile: default_lockfile(),
            production: false,
            buck: BuckConfig::default(),
            vendor: VendorConfig::default(),
            platform: HashMap::new(),
        }
    }
}

impl HerderConfig {
    pub fn load(dir: &str) -> Self {
        let config_path = Path::new(dir).join("herder.toml");
        if !config_path.exists() {
            return Self::default();
        }

        match std::fs::read_to_string(&config_path) {
            Ok(content) => match toml::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("Warning: failed to parse herder.toml: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("Warning: failed to read herder.toml: {}", e);
                Self::default()
            }
        }
    }
}
