use std::path::PathBuf;
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bind_host: String,
    pub bind_port: u16,
    pub plugins: BTreeMap<String, PluginConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginConfig {
    pub path: PathBuf,
}
