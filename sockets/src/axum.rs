
use crate::CloseCode;
use crate::CloseFrame;
use crate::RawMessage;
use crate::Server;
use crate::ServerExt;
use crate::Socket;
use async_trait::async_trait;
use axum::extract::ws;
use axum::extract::ws::rejection::*;
use axum::extract::ConnectInfo;
use axum::extract::FromRequest;
use axum::response::Response;
use std::net::SocketAddr;

/// Extractor for establishing WebSocket connections.
///
/// Note: This extractor requires the request method to be `GET` so it should
/// always be used with [`get`](axum::routing::get). Requests with other methods will be
/// rejected.
///
/// See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Upgrade {
    ws: ws::WebSocketUpgrade,
    address: SocketAddr,
    request: crate::Request,
}

#[async_trait]
impl<S, B> FromRequest<S, B> for Upgrade
where
    S: Send + Sync,
    B: Send + 'static,
{
    type Rejection = WebSocketUpgradeRejection;

    async fn from_request(req: http::Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let ConnectInfo(address) = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .expect("Axum Server must be created with `axum::Router::into_make_service_with_connect_info::<SocketAddr, _>()`")
            .to_owned();

        let mut pure_req = crate::Request::builder()
            .method(req.method())
            .uri(req.uri())
            .version(req.version());
        for (k, v) in req.headers() {
            pure_req = pure_req.header(k, v);
        }
        let pure_req = pure_req.body(()).unwrap();

        Ok(Self {
            ws: ws::WebSocketUpgrade::from_request(req, state).await?,
            address,
            request: pure_req,
        })
    }
}

impl From<ws::Message> for RawMessage {
    fn from(message: ws::Message) -> Self {
        match message {
            ws::Message::Text(text) => RawMessage::Text(text),
            ws::Message::Binary(binary) => RawMessage::Binary(binary),
            ws::Message::Ping(ping) => RawMessage::Ping(ping),
            ws::Message::Pong(pong) => RawMessage::Pong(pong),
            ws::Message::Close(Some(close)) => RawMessage::Close(Some(CloseFrame {
                code: CloseCode::try_from(close.code).unwrap(),
                reason: close.reason.into(),
            })),
            ws::Message::Close(None) => RawMessage::Close(None),
        }
    }
}

impl From<RawMessage> for ws::Message {
    fn from(message: RawMessage) -> Self {
        match message {
            RawMessage::Text(text) => ws::Message::Text(text),
            RawMessage::Binary(binary) => ws::Message::Binary(binary),
            RawMessage::Ping(ping) => ws::Message::Ping(ping),
            RawMessage::Pong(pong) => ws::Message::Pong(pong),
            RawMessage::Close(Some(close)) => ws::Message::Close(Some(ws::CloseFrame {
                code: close.code.into(),
                reason: close.reason.into(),
            })),
            RawMessage::Close(None) => ws::Message::Close(None),
        }
    }
}

impl Upgrade {
    /// Finalize upgrading the connection and call the provided callback with
    /// the stream.
    ///
    /// When using `WebSocketUpgrade`, the response produced by this method
    /// should be returned from the handler. See the [module docs](self) for an
    /// example.
    pub fn on_upgrade<E: ServerExt + 'static>(self, server: Server<E>) -> Response {
        self.ws.on_upgrade(move |socket| async move {
            let socket = Socket::new(socket, Default::default()); // TODO: Make it really configurable via Extensions
            server.accept(socket, self.request, self.address).await;
        })
    }
}