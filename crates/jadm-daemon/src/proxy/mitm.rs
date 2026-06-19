use anyhow::{Result, anyhow};
use rustls::server::Acceptor;
use tokio_rustls::LazyConfigAcceptor;
use std::sync::Arc;
use rustls::ServerConfig;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, body::Incoming};
use bytes::Bytes;
use http_body_util::{Full, Either};
use tokio::net::TcpStream;
use crate::proxy::ca::CertificateAuthority;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use hyper_util::rt::TokioIo;
use socket2::{Socket, Domain, Type, Protocol};

#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }
    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

pub async fn handle_connection(stream: TcpStream, ca: Arc<CertificateAuthority>, mark: u32) -> Result<()> {
    let acceptor = Acceptor::default();
    let start = LazyConfigAcceptor::new(acceptor, stream).await?;
    let client_hello = start.client_hello();
    
    let sni = client_hello.server_name().unwrap_or("localhost").to_string();
    
    let (cert, key_pair) = ca.generate_leaf_cert(&sni)?;
    
    let cert_der = cert.der().to_vec();
    let key_der = key_pair.serialize_der();
    
    let cert_chain = vec![CertificateDer::from(cert_der)];
    let private_key = PrivateKeyDer::Pkcs8(key_der.into());
    
    let mut server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)?;
        
    server_config.alpn_protocols = vec![b"http/1.1".to_vec()];
    let server_config = Arc::new(server_config);
    
    let tls_stream = start.into_stream(server_config).await?;
    let io = TokioIo::new(tls_stream);
    
    let service = service_fn(move |req| {
        let sni = sni.clone();
        async move {
            handle_request(req, sni, mark).await
        }
    });

    if let Err(err) = http1::Builder::new()
        .serve_connection(io, service)
        .with_upgrades()
        .await
    {
        eprintln!("Error serving connection: {:?}", err);
    }
    
    Ok(())
}

async fn handle_request(req: Request<Incoming>, sni: String, mark: u32) -> Result<Response<Either<Incoming, Full<Bytes>>>, anyhow::Error> {
    let host = req.uri().host().unwrap_or(&sni).to_string();
    let port = req.uri().port_u16().unwrap_or(443);
    
    // Use marked socket to bypass redirection
    let addr = tokio::net::lookup_host((&host as &str, port)).await?
        .next()
        .ok_or_else(|| anyhow!("Failed to resolve host"))?;

    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))?;
    socket.set_nonblocking(true)?;
    #[cfg(target_os = "linux")]
    socket.set_mark(mark)?;
    
    let std_stream: std::net::TcpStream = socket.into();
    let stream = match TcpStream::from_std(std_stream) {
        Ok(s) => s,
        Err(e) => return Err(anyhow::anyhow!("Failed to convert std TcpStream: {}", e)),
    };
    
    let config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(NoVerifier))
        .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
    let domain = ServerName::try_from(host.as_str()).map_err(|e| anyhow!("Invalid DNS name {}: {}", host, e))?.to_owned();
    
    let tls_stream = connector.connect(domain, stream).await?;
    let io = TokioIo::new(tls_stream);
    
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;
    tokio::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Upstream connection failed: {:?}", err);
        }
    });
    
    // Determine the full URL for JADM queue if intercepted
    let uri_string = format!("https://{}{}", host, req.uri().path_and_query().map(|pq| pq.as_str()).unwrap_or(""));
    
    let upstream_res = sender.send_request(req).await?;
    
    let is_attachment = upstream_res.headers().get("Content-Disposition")
        .map(|v| v.to_str().unwrap_or("").contains("attachment")).unwrap_or(false);
    let is_video = upstream_res.headers().get("Content-Type")
        .map(|v| v.to_str().unwrap_or("").starts_with("video/")).unwrap_or(false);

    if is_attachment || is_video {
        println!("Intercepted URL: {}", uri_string);
        // Intercepted
        let html = format!("<h1>Intercepted by JADM</h1><p>URL: {}</p>", uri_string);
        let res = Response::builder()
            .status(200)
            .header("Content-Type", "text/html")
            .body(Either::Right(Full::new(Bytes::from(html))))
            .unwrap();
        Ok(res)
    } else {
        // Stream back upstream response
        let (parts, body) = upstream_res.into_parts();
        let res = Response::from_parts(parts, Either::Left(body));
        Ok(res)
    }
}
