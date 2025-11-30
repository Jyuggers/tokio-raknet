mod offline;
mod online;

use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use bytes::Bytes;

use crate::protocol::constants::UDP_HEADER_SIZE;
use crate::transport::listener_conn::{RaknetConnection, SessionState};
use crate::transport::mux::new_tick_interval;

use offline::PendingConnection;
use online::{dispatch_datagram, handle_outgoing_msg, tick_sessions};

use super::OutboundMsg;

pub const MAX_PENDING_CONNECTIONS: usize = 1024;

/// Server-side RakNet listener that accepts new connections.
pub struct RaknetListener {
    local_addr: SocketAddr,
    new_connections: mpsc::Receiver<(
        SocketAddr,
        mpsc::Receiver<Result<Bytes, crate::RaknetError>>,
    )>,
    outbound_tx: mpsc::Sender<OutboundMsg>,
}

impl RaknetListener {
    /// Binds a new listener to the specified address.
    pub async fn bind(addr: SocketAddr, mtu: usize) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        let local_addr = socket.local_addr()?;

        let (new_conn_tx, new_conn_rx) = mpsc::channel(32);
        let (outbound_tx, outbound_rx) = mpsc::channel(1024);

        tokio::spawn(run_listener_muxer(
            socket,
            mtu,
            new_conn_tx,
            outbound_rx,
        ));

        Ok(Self {
            local_addr,
            new_connections: new_conn_rx,
            outbound_tx,
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Accepts the next incoming connection.
    pub async fn accept(&mut self) -> Option<RaknetConnection> {
        let (peer, incoming) = self.new_connections.recv().await?;
        Some(RaknetConnection {
            peer,
            incoming,
            outbound_tx: self.outbound_tx.clone(),
        })
    }
}

async fn run_listener_muxer(
    socket: UdpSocket,
    mtu: usize,
    new_conn_tx: mpsc::Sender<(
        SocketAddr,
        mpsc::Receiver<Result<Bytes, crate::RaknetError>>,
    )>,
    mut outbound_rx: mpsc::Receiver<OutboundMsg>,
) {
    let mut buf = vec![0u8; mtu + UDP_HEADER_SIZE + 64];
    let mut sessions: HashMap<SocketAddr, SessionState> = HashMap::new();
    let mut pending: HashMap<SocketAddr, PendingConnection> = HashMap::new();
    let mut tick = new_tick_interval();

    loop {
        tokio::select! {
            res = socket.recv_from(&mut buf) => {
                let Ok((len, peer)) = res else { break; };
                dispatch_datagram(
                    &socket,
                    mtu,
                    &buf[..len],
                    peer,
                    &mut sessions,
                    &mut pending,
                    &new_conn_tx,
                ).await;
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
