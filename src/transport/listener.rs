mod offline;
mod online;

use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::protocol::constants::UDP_HEADER_SIZE;
use crate::transport::listener_conn::SessionState;
use crate::transport::mux::new_tick_interval;
use crate::transport::stream::RaknetStream;
use std::sync::{Arc, RwLock};

use offline::PendingConnection;
use online::{dispatch_datagram, handle_outgoing_msg, tick_sessions};

use super::OutboundMsg;

pub const MAX_PENDING_CONNECTIONS: usize = 1024;

/// Server-side RakNet listener that accepts new connections.
pub struct RaknetListener {
    local_addr: SocketAddr,
    new_connections: mpsc::Receiver<(
        SocketAddr,
        mpsc::Receiver<Result<super::ReceivedMessage, crate::RaknetError>>,
    )>,
    outbound_tx: mpsc::Sender<OutboundMsg>,
    advertisement: Arc<RwLock<Vec<u8>>>,
}

impl RaknetListener {
    /// Binds a new listener to the specified address.
    pub async fn bind(addr: SocketAddr, mtu: usize) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        let local_addr = socket.local_addr()?;

        let (new_conn_tx, new_conn_rx) = mpsc::channel(32);
        let (outbound_tx, outbound_rx) = mpsc::channel(1024);
        let advertisement = Arc::new(RwLock::new(b"MCPE;Dedicated Server;527;1.19.1;0;10;13253860892328930865;Bedrock level;Survival;1;19132".to_vec()));

        tokio::spawn(run_listener_muxer(
            socket,
            mtu,
            new_conn_tx,
            outbound_rx,
            advertisement.clone(),
        ));

        Ok(Self {
            local_addr,
            new_connections: new_conn_rx,
            outbound_tx,
            advertisement,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Accepts the next incoming connection.
    pub async fn accept(&mut self) -> Option<RaknetStream> {
        let (peer, incoming) = self.new_connections.recv().await?;
        Some(RaknetStream::new(
            self.local_addr,
            peer,
            incoming,
            self.outbound_tx.clone(),
        ))
    }

    /// Sets the advertisement data (Pong payload) sent in response to UnconnectedPing (0x01) and OpenConnections (0x02).
    pub fn set_advertisement(&self, data: Vec<u8>) {
        if let Ok(mut guard) = self.advertisement.write() {
            *guard = data;
        }
    }

    /// Gets a copy of the current advertisement data.
    pub fn get_advertisement(&self) -> Vec<u8> {
        self.advertisement
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

async fn run_listener_muxer(
    socket: UdpSocket,
    mtu: usize,
    new_conn_tx: mpsc::Sender<(
        SocketAddr,
        mpsc::Receiver<Result<super::ReceivedMessage, crate::RaknetError>>,
    )>,
    mut outbound_rx: mpsc::Receiver<OutboundMsg>,
    advertisement: Arc<RwLock<Vec<u8>>>,
) {
    let mut buf = vec![0u8; mtu + UDP_HEADER_SIZE + 64];
    let mut sessions: HashMap<SocketAddr, SessionState> = HashMap::new();
    let mut pending: HashMap<SocketAddr, PendingConnection> = HashMap::new();
    let mut tick = new_tick_interval();

    loop {
        tokio::select! {
            res = socket.recv_from(&mut buf) => {
                match res  {
                    Ok((len, peer)) => {
                        dispatch_datagram(
                            &socket,
                            mtu,
                            &buf[..len],
                            peer,
                            &mut sessions,
                            &mut pending,
                            &new_conn_tx,
                            &advertisement,
                        ).await;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::ConnectionReset {
                            // Windows ICMP port unreachable - ignore
                            continue;
                        }
                        tracing::error!("UDP socket error: {}", e);
                        // Don't break on transient errors
                        continue;
                    }
                }
            }

            Some(msg) = outbound_rx.recv() => {
                handle_outgoing_msg(&socket, mtu, msg, &mut sessions).await;
            }

            _ = tick.tick() => {
                tick_sessions(&socket, &mut sessions).await;
            }
        }
    }
}
