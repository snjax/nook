use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::error::{NookError, NookResult};

/// Start a TCP proxy from host_port to container_ip:container_port
pub async fn start_port_proxy(
    host_port: u16,
    container_ip: String,
    container_port: u16,
    cancel: CancellationToken,
) -> NookResult<()> {
    let addr: SocketAddr = format!("0.0.0.0:{}", host_port)
        .parse()
        .map_err(|e| NookError::Other(format!("Invalid address: {}", e)))?;

    let listener = TcpListener::bind(addr).await.map_err(|e| {
        if e.kind() == std::io::ErrorKind::AddrInUse {
            NookError::HostPortBusy(host_port)
        } else {
            NookError::Io(e)
        }
    })?;

    tracing::info!(
        "Port proxy started: 0.0.0.0:{} → {}:{}",
        host_port,
        container_ip,
        container_port
    );

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                tracing::info!("Port proxy stopped for port {}", host_port);
                break;
            }
            result = listener.accept() => {
                match result {
                    Ok((client_stream, _)) => {
                        let target = format!("{}:{}", container_ip, container_port);
                        let cancel_clone = cancel.clone();
                        tokio::spawn(async move {
                            if let Err(e) = proxy_connection(client_stream, &target, cancel_clone).await {
                                tracing::debug!("Proxy connection error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to accept connection: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

async fn proxy_connection(
    mut client: tokio::net::TcpStream,
    target: &str,
    cancel: CancellationToken,
) -> NookResult<()> {
    let mut server = tokio::net::TcpStream::connect(target)
        .await
        .map_err(|e| NookError::Other(format!("Failed to connect to container: {}", e)))?;

    let (mut client_read, mut client_write) = client.split();
    let (mut server_read, mut server_write) = server.split();

    let mut client_buf = vec![0u8; 8192];
    let mut server_buf = vec![0u8; 8192];

    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            result = client_read.read(&mut client_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        if server_write.write_all(&client_buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            result = server_read.read(&mut server_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        if client_write.write_all(&server_buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }

    Ok(())
}
