use std::borrow::Cow;
use tokio_tungstenite::tungstenite;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message {
    Text(String),
    Binary(Vec<u8>),
    Ping(Vec<u8>),
    Pong(Vec<u8>),
    Close(Option<CloseFrame<'static>>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CloseFrame<'a> {
    pub code: u16,
    pub reason: Cow<'a, str>,
}

impl From<Message> for tungstenite::Message {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Text(text) => tungstenite::Message::Text(text),
            Message::Binary(data) => tungstenite::Message::Binary(data),
            Message::Ping(data) => tungstenite::Message::Ping(data),
            Message::Pong(data) => tungstenite::Message::Pong(data),
            Message::Close(Some(frame)) => {
                tungstenite::Message::Close(Some(tungstenite::protocol::CloseFrame {
                    code: frame.code.into(),
                    reason: frame.reason.into(),
                }))
            }
            Message::Close(None) => tungstenite::Message::Close(None),
        }
    }
} 