use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use anyhow::{Result, anyhow};

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub package: Package,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub profile: HashMap<String, Profile>,
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Package {
    pub name: String,
    pub _version: String,
    #[serde(default = "default_type")]
    pub _type: String,
}

fn default_type() -> String { "bin".to_string() }

#[derive(Debug, Deserialize, Default)]
pub struct Build {
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default)]
    pub lib_dirs: Vec<String>,
    #[serde(default)]
    pub libs: Vec<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Profile {
    #[serde(default)]
    pub flags: Vec<String>,
    pub opt_level: Option<u8>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum Dependency {
    Path { _path: String },
    System { system: bool, _version: Option<String> },
}

impl Config {
    pub fn load(project_root: &Path) -> Result<Self> {
        let config_path = project_root.join("cplus.toml");
        if !config_path.exists() {
            return Err(anyhow!("cplus.toml not found in {:?}", project_root));
        }
        let content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
