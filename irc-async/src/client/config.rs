/// Configuration for the IRC client
pub struct Config {
    /// The hostname to connect to
    pub host: String,

    /// The port of the IRC server
    pub port: u16,

    /// Whether or not to enable SSL
    pub ssl: bool,

    /// The nick to connect with
    pub nick: String,
}
