use super::channels::{self, DialogueQueueParams, RunnerStateInfo};
use super::error;
use super::runtimes::*;
use super::task::run_task;

use ws_message::{ClientMessage, ServerMessage};

use axum::extract::State;
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;
use axum::{
    Router,
    extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
};
use axum_extra::{TypedHeader, headers};
use futures_util::{sink::SinkExt, stream::StreamExt};
use tokio::sync::Mutex;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};

use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::ControlFlow;
use std::sync::Arc;

pub async fn run(addr: impl ToSocketAddrs, runtimes: Runtimes) -> error::Result<()> {
    let server = DiavoloServer::new();
    let runtime_paths = runtimes.keys().cloned().collect();
    let _res = tokio::try_join!(run_task(runtimes), server.serve(addr, runtime_paths))?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    runner_state_info_rx: Arc<Mutex<tokio::sync::mpsc::Receiver<RunnerStateInfo>>>,
}

struct DiavoloServer {
    state: AppState,
}

impl DiavoloServer {
    fn new() -> Self {
        DiavoloServer {
            state: AppState {
                runner_state_info_rx: Arc::new(Mutex::new(channels::runner_state_info_rx())),
            },
        }
    }

    async fn serve(
        &self,
        addr: impl ToSocketAddrs,
        runtime_paths: Vec<RuntimePath>,
    ) -> error::Result<()> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .expect("could not parse address");
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("Diavolo Server Listening on {:?}", addr);

        axum::serve(
            listener,
            self.app(runtime_paths)
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await?;

        Ok(())
    }

    fn app(&self, runtime_paths: Vec<RuntimePath>) -> Router {
        let router = Router::new().layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

        runtime_paths
            .into_iter()
            .fold(router, |router, path| {
                router.route(path.as_ref(), any(Self::ws_handler))
            })
            .with_state(self.state.clone())
    }

    async fn ws_handler(
        ws: WebSocketUpgrade,
        uri: axum::http::Uri,
        user_agent: Option<TypedHeader<headers::UserAgent>>,
        ConnectInfo(addr): ConnectInfo<SocketAddr>,
        State(app_state): State<AppState>,
    ) -> impl IntoResponse {
        let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
            user_agent.to_string()
        } else {
            String::from("Unknown user agent")
        };
        tracing::debug!("WebSocket connection request for `{}`", uri.path());
        tracing::debug!("`{user_agent}` at {addr} connected.");
        ws.on_upgrade(move |socket| Self::handle_socket(socket, addr, uri, app_state))
    }

    async fn handle_socket(
        socket: WebSocket,
        who: SocketAddr,
        uri: axum::http::Uri,
        app_state: AppState,
    ) {
        tracing::debug!("Websocket context {who} created");

        let (mut sender, mut receiver) = socket.split();

        let mut send_task = tokio::spawn(async move {
            tracing::debug!("Send task started for {who}");
            let mut rx = app_state.runner_state_info_rx.lock().await;

            while let Some(runner_state_info) = rx.recv().await {
                let server_message = ServerMessage::Mutation;
                let _ = sender
                    .send(Message::Text(server_message.to_string().into()))
                    .await;
            }

            tracing::debug!("Sending close to {who}...");

            if let Err(e) = sender
                .send(Message::Close(Some(CloseFrame {
                    code: axum::extract::ws::close_code::NORMAL,
                    reason: Utf8Bytes::from_static("Goodbye"),
                })))
                .await
            {
                tracing::debug!("Could not send Close due to {e}, probably it is ok?");
            }
        });

        let mut recv_task = tokio::spawn(async move {
            tracing::debug!("Receive task started for {who}");

            loop {
                let msg = match receiver.next().await {
                    Some(Ok(msg)) => msg,
                    Some(Err(e)) => {
                        tracing::error!("Error receiving message from {who}: {}", e);
                        break;
                    }
                    None => {
                        tracing::debug!("WebSocket stream closed by {who}");
                        break;
                    }
                };

                match Self::process_message(msg, who, uri.path()).await {
                    ControlFlow::Continue(()) => {}
                    ControlFlow::Break(reason) => {
                        tracing::debug!("Breaking receive loop for {who} due to {:?}", reason);
                        break;
                    }
                }
            }

            tracing::debug!("Receiver loop ended for {who}");
        });

        tokio::select! {
            _ = &mut send_task => {
                tracing::debug!("Send task ended for {who}");
                recv_task.abort();
            }
            _ = &mut recv_task => {
                tracing::debug!("Receive task ended for {who}");
                send_task.abort();
            }
        }

        // returning from the handler closes the websocket connection
        tracing::debug!("Websocket context {who} destroyed");
    }

    async fn process_message(
        msg: Message,
        who: SocketAddr,
        path: &str,
    ) -> ControlFlow<BreakReason, ()> {
        match msg {
            Message::Text(t) => {
                let Ok(client_message) = t.as_str().parse::<ClientMessage>() else {
                    tracing::warn!(">>> {who} sent invalid text message: {t}");
                    return ControlFlow::Break(BreakReason::Error(
                        crate::error::Error::BoxedError("Invalid client message format".into()),
                    ));
                };

                tracing::debug!(">>> {who} sent client message: {client_message}");

                match client_message {
                    ClientMessage::Request(cr) => {
                        tracing::debug!("Client requested dialogue start");

                        let params = match DialogueQueueParams::from_request(path, cr).await {
                            Ok(params) => params,
                            Err(e) => {
                                tracing::error!("Failed to create dialogue queue params: {}", e);
                                return ControlFlow::Break(BreakReason::Error(e));
                            }
                        };

                        if let Err(e) = channels::send_dialogue_queue(params).await {
                            tracing::error!("Failed to send dialogue queue params: {}", e);
                            return ControlFlow::Break(BreakReason::Error(e.into()));
                        }
                    }
                    ClientMessage::Mutation => {
                        tracing::debug!("Client requested mutation");
                    }
                    ClientMessage::Cancel => {
                        tracing::debug!("Client requested cancellation");
                    }
                }
            }
            Message::Binary(d) => {
                tracing::warn!(">>> {who} sent binary data: {d:?}");
                ControlFlow::Break(BreakReason::Error(error::Error::BoxedError(
                    "Binary messages are not supported".into(),
                )))?;
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
}

#[derive(Debug)]
enum BreakReason {
    ClosedByClient(Option<CloseFrame>),
    Error(crate::error::Error),
}
