use futures_util::{
    sink::Sink,
    stream::Stream,
};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use pin_project_lite::pin_project;
use std::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};
use tokio_tungstenite::{tungstenite, WebSocketStream};

pub mod upgrade;

mod wasm;
pub use self::wasm::{CloseFrame, Message};

pin_project! {
    pub struct WebSocket {
        #[pin]
        inner: WebSocketStream<TokioIo<Upgraded>>,
    }
}

impl WebSocket {
    pub(crate) fn new(stream: WebSocketStream<TokioIo<Upgraded>>) -> Self {
        Self { inner: stream }
    }
}

impl Stream for WebSocket {
    type Item = Result<Message, tungstenite::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        match futures_util::ready!(this.inner.poll_next(cx)) {
            Some(Ok(msg)) => Poll::Ready(Some(Ok(msg.into()))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

impl Sink<Message> for WebSocket {
    type Error = tungstenite::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.project().inner.start_send(item.into())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl fmt::Debug for WebSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WebSocket").finish()
    }
} 