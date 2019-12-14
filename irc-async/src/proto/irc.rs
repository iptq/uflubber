//! Implementation of IRC codec for Tokio.
use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder, LinesCodec};

use super::errors::IrcError;
use super::message::Message;

/// An IRC codec built around an inner codec.
#[derive(Default)]
pub struct IrcCodec {
    inner: LinesCodec,
}

impl IrcCodec {
    /// Sanitizes the input string by cutting up to (and including) the first occurence of a line
    /// terminiating phrase (`\r\n`, `\r`, or `\n`). This is used in sending messages back to
    /// prevent the injection of additional commands.
    pub(crate) fn sanitize(mut data: String) -> String {
        // n.b. ordering matters here to prefer "\r\n" over "\r"
        if let Some((pos, len)) = ["\r\n", "\r", "\n"]
            .iter()
            .flat_map(|needle| data.find(needle).map(|pos| (pos, needle.len())))
            .min_by_key(|&(pos, _)| pos)
        {
            data.truncate(pos + len);
        }
        data
    }
}

impl Decoder for IrcCodec {
    type Item = Message;
    type Error = IrcError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Message>, Self::Error> {
        let result = self
            .inner
            .decode(src)
            .map_err(IrcError::from)
            .and_then(|res| res.map_or(Ok(None), |msg| msg.parse::<Message>().map(Some)));
        result
    }
}

impl Encoder for IrcCodec {
    type Item = Message;
    type Error = IrcError;

    fn encode(&mut self, msg: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.inner
            .encode(IrcCodec::sanitize(msg.to_string()), dst)
            .map_err(IrcError::from)
    }
}
