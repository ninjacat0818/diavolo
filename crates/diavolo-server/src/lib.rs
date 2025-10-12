mod manager;

use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;
use axum::{
    Router,
    body::Bytes,
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
};
use axum_extra::{TypedHeader, headers};

use std::hash::Hash;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::str::FromStr;

use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use futures_util::{sink::SinkExt, stream::StreamExt};

pub fn get_app() -> Router {
    Router::new().route("/", any(ws_handler)).layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    )
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown user agent")
    };
    tracing::debug!("`{user_agent}` at {addr} connected.");
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    if socket.send(Message::Ping(Bytes::new())).await.is_ok() {
        tracing::debug!("Pinged {who}...");
    } else {
        tracing::debug!("Could not send ping {who}!");
        return;
    }

    let (mut sender, mut receiver) = socket.split();

    // let mut send_task = tokio::spawn(async move {
    //     loop {
    //         tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    //     }

    //     tracing::debug!("Sending close to {who}...");

    //     if let Err(e) = sender
    //         .send(Message::Close(Some(CloseFrame {
    //             code: axum::extract::ws::close_code::NORMAL,
    //             reason: Utf8Bytes::from_static("Goodbye"),
    //         })))
    //         .await
    //     {
    //         tracing::debug!("Could not send Close due to {e}, probably it is ok?");
    //     }
    // });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let ControlFlow::Break(reason) = process_message(msg, who) {
                return;
            }
        }
    });

    // returning from the handler closes the websocket connection
    tracing::debug!("Websocket context {who} destroyed");
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<BreakReason, ()> {
    match msg {
        Message::Text(t) => {
            if let Ok(client_message) = t.as_str().parse::<ClientMessage>() {
                tracing::debug!(">>> {who} sent client message: {client_message}");
            } else {
                tracing::warn!(">>> {who} sent invalid text message: {t}");
                let err = Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid client message format",
                ));
                ControlFlow::Break(BreakReason::Error(err))?;
            }
        }
        Message::Binary(d) => {
            tracing::warn!(">>> {who} sent binary data: {d:?}");
            let err = Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Binary messages are not supported",
            ));
            ControlFlow::Break(BreakReason::Error(err))?;
        }
        Message::Close(c) => {
            if let Some(cf) = &c {
                tracing::debug!(
                    ">>> {who} sent close with code {} and reason `{}`",
                    cf.code,
                    cf.reason
                )
            } else {
                tracing::debug!(">>> {who} sent close message without CloseFrame");
            }

            ControlFlow::Break(BreakReason::ClosedByClient(c))?;
        }

        Message::Pong(v) => {
            tracing::debug!(">>> {who} sent pong with {v:?}");
        }
        Message::Ping(v) => {
            tracing::debug!(">>> {who} sent ping with {v:?}");
        }
    }
    ControlFlow::Continue(())
}

enum BreakReason {
    ClosedByClient(Option<axum::extract::ws::CloseFrame>),
    Error(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use crate::ClientMessage;

    use super::get_app;
    use axum::extract::ConnectInfo;
    use axum_test::TestServer;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn it_should_work() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .try_init()
            .ok();

        let test_addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
        let app = get_app().layer(axum::Extension(ConnectInfo(test_addr)));

        let server = TestServer::builder().http_transport().build(app).unwrap();
        let mut websocket = server.get_websocket("/").await.into_websocket().await;

        let client_message: ClientMessage = serde_json::json!({
            "type": "request",
            "endpoint": "/dialogue_script.yml",
            "args": {
                "x": "Hello, Diavolo!",
                "y": 42
            }
        })
        .try_into()
        .unwrap();

        websocket.send_json(&client_message).await;
        websocket.close().await;
    }
}

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum ClientMessage {
    Request {
        endpoint: Endpoint,
        args: serde_json::Value,
    },
    Mutation,
    Cancel,
}

impl std::fmt::Display for ClientMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_string(self).expect("Failed to serialize ClientMessage");
        write!(f, "{}", s)
    }
}

impl TryFrom<serde_json::Value> for ClientMessage {
    type Error = serde_json::Error;

    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(value)
    }
}

impl FromStr for ClientMessage {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Mutation,
    Terminated,
    Error,
}

pub enum Endpoint {
    Url(url::Url),
    Path(std::path::PathBuf),
}

impl Serialize for Endpoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Endpoint::Url(url) => serializer.serialize_str(url.as_str()),
            Endpoint::Path(path) => serializer.serialize_str(path.to_str().unwrap_or_default()),
        }
    }
}

impl<'de> Deserialize<'de> for Endpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if let Ok(url) = url::Url::parse(&s) {
            Ok(Endpoint::Url(url))
        } else {
            Ok(Endpoint::Path(std::path::PathBuf::from(s)))
        }
    }
}

#[cfg(test)]
mod message_tests {
    use super::{ClientMessage, ServerMessage};

    #[test]
    fn serde_client_message() {
        let deserialized: ClientMessage = serde_json::json!({
            "type": "request",
            "endpoint": "/dialogue_script.yml",
            "args": {
                "x": 42,
                "y": "Hello, Diavolo!"
            },
        })
        .try_into()
        .unwrap();

        deserialized.to_string().parse::<ClientMessage>().unwrap();
    }
}
