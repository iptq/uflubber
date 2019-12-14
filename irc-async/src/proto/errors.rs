use std::io;

use tokio_util::codec::LinesCodecError;

/// The main crate-wide error type.
#[derive(Debug, Error)]
pub enum IrcError {
    /// An internal I/O error.
    #[error("an io error occurred")]
    Io(#[from] io::Error),

    // /// An internal TLS error.
    // #[error("a TLS error occurred")]
    // Tls(#[source] TlsError),

    // /// An internal synchronous channel closed.
    // #[error("a sync channel closed")]
    // SyncChannelClosed(#[source] RecvError),

    // /// An internal asynchronous channel closed.
    // #[error("an async channel closed")]
    // AsyncChannelClosed(#[source] SendError<Message>),

    // /// An internal oneshot channel closed.
    // #[error("a oneshot channel closed")]
    // OneShotCanceled(#[source] Canceled),

    // /// An internal timer error.
    // #[error("timer failed")]
    // Timer(#[source] TimerError),

    // /// Error for invalid configurations.
    // #[error("invalid config: {}", path)]
    // InvalidConfig {
    //     /// The path to the configuration, or "<none>" if none specified.
    //     path: String,
    //     /// The detailed configuration error.
    //     #[source]
    //     cause: ConfigError,
    // },
    /// Error for invalid messages.
    #[error("invalid message: {string}")]
    InvalidMessage {
        /// The string that failed to parse.
        string: String,
        /// The detailed message parsing error.
        #[source]
        cause: MessageParseError,
    },
    // /// Mutex for a logged transport was poisoned making the log inaccessible.
    // #[error("mutex for a logged transport was poisoned")]
    // PoisonedLog,

    // /// Ping timed out due to no response.
    // #[error("connection reset: no ping response")]
    // PingTimeout,

    // /// Failed to lookup an unknown codec.
    // #[error("unknown codec: {}", codec)]
    // UnknownCodec {
    //     /// The attempted codec.
    //     codec: String,
    // },

    // /// Failed to encode or decode something with the given codec.
    // #[error("codec {} failed: {}", codec, data)]
    // CodecFailed {
    //     /// The canonical codec name.
    //     codec: &'static str,
    //     /// The data that failed to encode or decode.
    //     data: String,
    // },
    /// Failed to encode or decode a line
    #[error("line codec failed: {0}")]
    Codec(#[from] LinesCodecError),
    // /// All specified nicknames were in use or unusable.
    // #[error("none of the specified nicknames were usable")]
    // NoUsableNick,

    // /// This allows you to produce any `failure::Error` within closures used by
    // /// the irc crate. No errors of this kind will ever be produced by the crate
    // /// itself.
    // #[error("{}", inner)]
    // Custom {
    //     /// The actual error that occurred.
    //     inner: failure::Error
    // },
}

/// Errors that occur while parsing mode strings.
#[derive(Debug, Error)]
pub enum ModeParseError {
    /// Invalid modifier used in a mode string (only + and - are valid).
    #[error("invalid mode modifier: {modifier}")]
    InvalidModeModifier {
        /// The invalid mode modifier.
        modifier: char,
    },

    /// Missing modifier used in a mode string.
    #[error("missing mode modifier")]
    MissingModeModifier,
}

/// Errors that occur when parsing messages.
#[derive(Debug, Error)]
pub enum MessageParseError {
    /// The message was empty.
    #[error("empty message")]
    EmptyMessage,

    /// The command was invalid (i.e. missing).
    #[error("invalid command")]
    InvalidCommand,

    /// The mode string was malformed.
    #[error("invalid mode string: {string}")]
    InvalidModeString {
        /// The invalid mode string.
        string: String,
        /// The detailed mode parsing error.
        #[source]
        cause: ModeParseError,
    },

    /// The subcommand used was invalid.
    #[error("invalid {cmd} subcommand: {sub}")]
    InvalidSubcommand {
        /// The command whose invalid subcommand was referenced.
        cmd: &'static str,
        /// The invalid subcommand.
        sub: String,
    },
}
