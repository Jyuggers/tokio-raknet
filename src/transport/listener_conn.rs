use bytes::Bytes;
use tokio::sync::mpsc;

use crate::protocol::packet::RaknetPacket;
use crate::session::manager::ManagedSession;
use crate::transport::OutboundMsg;

/// Server-side connection handle returned from `RaknetListener::accept`.
pub struct RaknetConnection {
    /// Remote peer address for this connection.
    pub peer: std::net::SocketAddr,
    pub(crate) incoming: mpsc::Receiver<Result<Bytes, crate::RaknetError>>,
    pub(crate) outbound_tx: mpsc::Sender<OutboundMsg>,
}

impl RaknetConnection {
    pub fn peer_addr(&self) -> std::net::SocketAddr {
        self.peer
    }

    pub async fn recv(&mut self) -> Option<Result<Bytes, crate::RaknetError>> {
        self.incoming.recv().await
    }

    pub async fn send(
        &self,
        msg: impl Into<super::Message>,
    ) -> Result<(), crate::RaknetError> {
        let msg = msg.into();
        let payload = msg.buffer;

        if payload.is_empty() {
            return Ok(());
        }
        let id = payload[0];
        let body = payload.slice(1..);
        self.outbound_tx
            .send(OutboundMsg {
                peer: self.peer,
                packet: RaknetPacket::UserData { id, payload: body },
                reliability: msg.reliability,
                channel: msg.channel,
                priority: msg.priority,
            })
            .await
            .map_err(|_| crate::RaknetError::ConnectionClosed)
    }
}

/// Internal per-peer session state.
pub struct SessionState {
    pub managed: ManagedSession,
    pub to_app: mpsc::Sender<Result<Bytes, crate::RaknetError>>,
    pub pending_rx: Option<mpsc::Receiver<Result<Bytes, crate::RaknetError>>>,
    pub announced: bool,
}
