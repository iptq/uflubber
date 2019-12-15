use crate::backend::{RoomID, NewMessage};

pub enum Update {}

pub struct Request {
    pub sequence_number: u32,
    pub body: RequestBody,
}

pub enum RequestBody {
    RoomJoin(RoomID),
    MessageSend(NewMessage),
}
