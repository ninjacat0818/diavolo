pub(crate) mod channels;
pub(crate) mod error;
pub(crate) mod runtimes;
pub(crate) mod server;
pub(crate) mod task;

pub use runtimes::{Runtime, RuntimePath, Runtimes};
pub use server::*;

#[cfg(test)]
mod tests {
    use super::*;
    use ws_message::ClientRequest;

    async fn wait_for_server(
        addr: &str,
        max_attempts: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for _attempt in 0..max_attempts {
            if tokio::net::TcpStream::connect(addr).await.is_ok() {
                return Ok(());
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        Err("Server did not start in time".into())
    }

    #[tokio::test]
    async fn it_works() {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();

        const RUNTIME_PATH: &str = "/runtime_path";
        let socket_addr = "127.0.0.1:8080".parse::<std::net::SocketAddr>().unwrap();
        let socket_addr_string = socket_addr.to_string();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let mut runtimes = Runtimes::default();
                runtimes.insert(RuntimePath::from(RUNTIME_PATH), Runtime::default());

                let _ = crate::run(socket_addr, runtimes).await;
            });
        });

        wait_for_server(&socket_addr_string, 20).await.unwrap();

        let (mut client, receiver) =
            diavolo_client::connect(&format!("ws://{}{}", &socket_addr_string, RUNTIME_PATH))
                .await
                .unwrap();

        let request = ClientRequest {
            endpoint: "tests/fixtures/dialogue.yml".into(),
            actors: serde_json::json!([]),
            args: serde_json::json!({
                // "x": "Hello, Diavolo!",
                // "y": 42
            }),
        };

        client.send_request(request).await.unwrap();

        // channels::send_op(op).await?;

        tokio::time::sleep(tokio::time::Duration::from_millis(100000)).await;
    }
}
