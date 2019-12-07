use std::pin::Pin;

use anyhow::Error;
use bytes::{BytesMut,};
use serde_json::Value;
use tokio_serde::Deserializer;

pub struct JsonCodec;

impl Deserializer<Value> for JsonCodec {
    type Error = Error;

    fn deserialize(self: Pin<&mut Self>, bytes: &BytesMut) -> Result<Value, Self::Error> {
        serde_json::from_slice(bytes.as_ref()).map_err(Error::from)
    }
}