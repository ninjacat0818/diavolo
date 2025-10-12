use ws_message::{ClientMessage, ClientRequest, ServerMessage};

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::{self, Message, client::IntoClientRequest};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub async fn connect(
    request: impl IntoClientRequest + Unpin,
) -> Result<(DiavoloClient, ClientReceiver), tungstenite::Error> {
    let (ws_stream, response) = tokio_tungstenite::connect_async(request).await?;
    tracing::debug!("Response HTTP code: {}", response.status());

    let (write, read) = ws_stream.split();
    Ok((DiavoloClient { write }, ClientReceiver { read }))
}

pub struct DiavoloClient {
    write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl DiavoloClient {
    pub async fn send_request(&mut self, request: ClientRequest) -> tungstenite::Result<()> {
        let text = ClientMessage::Request(request).to_string().into();
        self.write.send(Message::Text(text)).await
    }
}

pub struct ClientReceiver {
    read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl ClientReceiver {
    pub async fn receive(
        &mut self,
        handler: impl AsyncFn(ServerMessage),
    ) -> Result<(), Box<dyn std::error::Error>> {
        while let Some(msg) = self.read.next().await {
            let msg: Message = msg?;

            match msg {
                Message::Text(text) => handler(text.as_str().parse()?).await,
                Message::Close(_) => {
                    tracing::debug!("Connection closed by server");
                }
                Message::Binary(bin) => {
                    return Err(format!("Received binary message: {:?}", bin).into());
                }
                Message::Ping(payload) => {
                    tracing::debug!("Received ping: {:?}", payload);
                }
                Message::Pong(payload) => {
                    tracing::debug!("Received pong: {:?}", payload);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
