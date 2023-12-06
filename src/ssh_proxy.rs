use makiko::{self, bytes::BytesMut, Client, Tunnel, TunnelReceiver};
use std::{
    io,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use tokio::{
    self,
    io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
    net::{TcpListener, TcpStream},
};

use crate::{config::SSHProxyConfig, PERMITS};
#[derive(Clone)]
pub struct SSHProxy {
    name: Option<String>,
    ssh_client: Client,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
}

impl SSHProxy {
    pub(crate) async fn new(client: Client, config: &SSHProxyConfig) -> Self {
        // TODO : better error
        Self {
            name: Some(config.name.clone()),
            ssh_client: client,
            local_addr: (
                IpAddr::from_str(&config.local_ip).unwrap(),
                config.local_port,
            )
                .into(),
            remote_addr: (
                IpAddr::from_str(&config.service_ip).unwrap(),
                config.service_port,
            )
                .into(),
        }
    }

    pub(crate) async fn start(&self) -> io::Result<()> {
        let listener = TcpListener::bind(self.local_addr).await?;
        log::debug!(
            "Starting proxy  {}  listening loop {:?}",
            &self.name.as_ref().unwrap_or(&"".to_string()),
            &self.local_addr
        );
        loop {
            let (socket, _) = listener.accept().await?;
            // TODO: this handle will fail if client disconnects
            tokio::spawn(handle(self.ssh_client.clone(), socket, self.remote_addr));
        }
    }
}

async fn tunnel_to_socket(
    mut tunnel_rx: TunnelReceiver,
    mut writer: WriteHalf<TcpStream>,
) -> io::Result<()> {
    while let Ok(Some(event)) = tunnel_rx.recv().await {
        match event {
            // Handle data received from the tunnel.
            makiko::TunnelEvent::Data(mut data) => {
                log::debug!("received data in tunnel");
                writer.write_all_buf(&mut data).await?;
            }

            // Handle EOF from the tunnel.
            makiko::TunnelEvent::Eof => {
                log::debug!("received tunnel EOF. Closing tunnel loop.");
                writer.shutdown().await?;
                break;
            }
            _ => unreachable!(),
        }
    }

    Ok(())
}
async fn socket_to_tunnel(mut socket_rd: ReadHalf<TcpStream>, tunnel: Tunnel) -> io::Result<()> {
    // Wait for the socket to be readable
    let mut buf = BytesMut::with_capacity(4096);
    loop {
        match socket_rd.read_buf(&mut buf).await {
            Ok(0) => {
                tunnel.send_eof().await.map_err(|e| {
                    log::error!("Tunnel send_eof error: {:?}", e);
                    io::ErrorKind::UnexpectedEof
                })?;
                break;
            }
            Ok(n) => {
                log::debug!("received {} bytesclient data.", n);
                // TODO : Zero-copy if possible ?
                let _ = tunnel.send_data(buf.split().freeze()).await.map_err(|e| {
                    log::error!("Tunnel send_data error: {:?}", e);
                    io::ErrorKind::InvalidInput
                })?;
                // TODO : select on this here
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                log::error!("Error client side: {:?}", e);
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    Ok(())
}

async fn handle(ssh_client: Client, socket: TcpStream, connect_addr: SocketAddr) -> io::Result<()> {
    // let connect_addr = ("localhost".into(), 8181);
    let origin_addr = socket.peer_addr()?;
    let origin_addr: (String, u16) = (origin_addr.ip().to_string(), origin_addr.port());

    // Open a tunnel from the server.
    let channel_config = makiko::ChannelConfig::default();

    log::debug!(
        "opening local port forwarding tunnel from {:?} -> {:?}",
        connect_addr,
        origin_addr
    );
    let (tunnel, tunnel_rx) = ssh_client
        .connect_tunnel(
            channel_config,
            (connect_addr.ip().to_string(), connect_addr.port()),
            origin_addr,
        )
        .await
        .map_err(|e| {
            log::error!(
                "Can't connect tunnel for peer:{:?}, err:{:?}",
                socket.peer_addr(),
                e
            );
            io::ErrorKind::NotConnected
        })?;
    let (socket_rd, socket_wr) = tokio::io::split(socket);
    // OpenSSH has a hard coded limit of 10_000 opened channels
    log::debug!("acquiring permit to open tunnel.");
    let permit = PERMITS.acquire().await.unwrap();

    let tunnel_to_socket_task = tokio::task::spawn(tunnel_to_socket(tunnel_rx, socket_wr));
    let socket_to_tunnnel_task = tokio::task::spawn(socket_to_tunnel(socket_rd, tunnel));

    // Wait for tunnel
    let _ = tokio::try_join!(tunnel_to_socket_task, socket_to_tunnnel_task)?;

    drop(permit);
    log::debug!("dropped channel_open permit");
    Ok(())
}
