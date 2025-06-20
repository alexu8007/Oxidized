use crate::{
    http_request::Request,
    response::Response,
    ws::{CloseFrame, Message, WebSocket},
    Result,
};
use base64::{engine::general_purpose, Engine as _};
use futures_util::Future;
use hyper::{
    self,
    header::{CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE},
    StatusCode,
};
use hyper_util::rt::TokioIo;
use sha1::{Digest, Sha1};
use tokio_tungstenite::{
    tungstenite::{self, protocol::Role},
    WebSocketStream,
};

pub fn upgrade<F, Fut>(f: F) -> impl Fn(Request) -> Result<Response>
where
    F: Fn(WebSocket) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    move |mut req: Request| {
        let key = req.inner().headers().get(SEC_WEBSOCKET_KEY).cloned();

        let f = f.clone();
        tokio::spawn(async move {
            match hyper::upgrade::on(req.inner_mut()).await {
                Ok(upgraded) => {
                    let io = TokioIo::new(upgraded);
                    let stream = WebSocketStream::from_raw_socket(io, Role::Server, None).await;
                    let ws = WebSocket::new(stream);
                    f(ws).await;
                }
                Err(e) => {
                    eprintln!("websocket upgrade error: {}", e);
                }
            }
        });

        let mut res = Response::new("");
        *res.inner_mut().status_mut() = StatusCode::SWITCHING_PROTOCOLS;

        if let Some(key) = key {
            let mut hasher = Sha1::new();
            hasher.update(key.as_bytes());
            hasher.update(b"258EAFA5-E914-47DA-95CA-C5AB0DC85B11"); // WebSocket magic string
            let accept_key = general_purpose::STANDARD.encode(hasher.finalize());
            res.inner_mut()
                .headers_mut()
                .insert(SEC_WEBSOCKET_ACCEPT, accept_key.parse().unwrap());
        }

        res.inner_mut()
            .headers_mut()
            .insert(CONNECTION, "upgrade".parse().unwrap());
        res.inner_mut()
            .headers_mut()
            .insert(UPGRADE, "websocket".parse().unwrap());

        Ok(res)
    }
}

impl From<tungstenite::Message> for Message {
    fn from(msg: tungstenite::Message) -> Self {
        match msg {
            tungstenite::Message::Text(text) => Message::Text(text),
            tungstenite::Message::Binary(data) => Message::Binary(data),
            tungstenite::Message::Ping(data) => Message::Ping(data),
            tungstenite::Message::Pong(data) => Message::Pong(data),
            tungstenite::Message::Close(Some(frame)) => Message::Close(Some(CloseFrame {
                code: frame.code.into(),
                reason: frame.reason.into(),
            })),
            tungstenite::Message::Close(None) => Message::Close(None),
            tungstenite::Message::Frame(_) => unimplemented!(),
        }
    }
} 