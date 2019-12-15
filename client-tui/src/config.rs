#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
}
