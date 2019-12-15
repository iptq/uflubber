use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub bind_host: String,
    pub bind_port: u16,
    pub backends: BTreeMap<String, BackendConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackendConfig {
    pub path: PathBuf,
}
