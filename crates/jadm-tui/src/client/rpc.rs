#[cfg(unix)]
use tokio::net::UnixStream;
#[cfg(not(unix))]
use tokio::net::TcpStream;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use jadm_common::protocol::{Request, Response};
use anyhow::{Result, anyhow};

pub struct RpcClient {
    socket_path: String,
}

impl RpcClient {
    pub fn new(socket_path: String) -> Self {
        Self { socket_path }
    }

    pub async fn send(&self, request: Request) -> Result<Response> {
        #[cfg(unix)]
        let mut stream = UnixStream::connect(&self.socket_path).await?;
        #[cfg(not(unix))]
        let mut stream = TcpStream::connect("127.0.0.1:6245").await?;

        let mut req_data = serde_json::to_vec(&request)?;
        req_data.push(b'\n');
        stream.write_all(&req_data).await?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line.is_empty() {
            return Err(anyhow!("No response from daemon"));
        }

        let response: Response = serde_json::from_str(&line)?;
        Ok(response)
    }
}
