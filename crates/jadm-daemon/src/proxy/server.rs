use anyhow::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::proxy::ca::CertificateAuthority;
use crate::proxy::mitm;

pub struct ProxyServer {
    port: u16,
    ca: Arc<CertificateAuthority>,
    mark: u32,
}

impl ProxyServer {
    pub fn new(port: u16, ca: Arc<CertificateAuthority>, mark: u32) -> Self {
        Self { port, ca, mark }
    }

    pub async fn run(&self) -> Result<()> {
        let listener = TcpListener::bind(("127.0.0.1", self.port)).await?;
        println!("Transparent TLS MITM Proxy listening on 127.0.0.1:{}", self.port);

        loop {
            let (stream, addr) = listener.accept().await?;
            let ca = self.ca.clone();
            let mark = self.mark;
            
            tokio::spawn(async move {
                if let Err(e) = mitm::handle_connection(stream, ca, mark).await {
                    eprintln!("MITM error for {}: {:?}", addr, e);
                }
            });
        }
    }
}
