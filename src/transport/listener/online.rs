use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::protocol::{datagram::Datagram, packet::RaknetPacket};
use crate::session::manager::{ConnectionState, ManagedSession};
use crate::transport::listener_conn::SessionState;
use crate::transport::mux::flush_managed;
use bytes::{BufMut, Bytes};

use super::offline::{
    PendingConnection, handle_offline, is_offline_packet_id, server_session_config,
};

pub(super) async fn dispatch_datagram(
    socket: &UdpSocket,
    mtu: usize,
    bytes: &[u8],
    peer: SocketAddr,
    sessions: &mut HashMap<SocketAddr, SessionState>,
    pending: &mut HashMap<SocketAddr, PendingConnection>,
    new_conn_tx: &mpsc::Sender<(
        SocketAddr,
        mpsc::Receiver<Result<Bytes, crate::RaknetError>>,
    )>,
) {
    if sessions.contains_key(&peer) {
        if !handle_incoming_udp(socket, mtu, bytes, peer, sessions, pending, new_conn_tx).await {
            // If decoding failed, drop the session to let the peer retry the handshake cleanly.
            sessions.remove(&peer);
            handle_offline(socket, mtu, bytes, peer, sessions, pending, new_conn_tx).await;
        }
        return;
    }

    if bytes.is_empty() {
        return;
    }

    if is_offline_packet_id(bytes[0]) {
        handle_offline(socket, mtu, bytes, peer, sessions, pending, new_conn_tx).await;
    } else {
        // Unexpected packet from unknown peer; ignore.
    }
}

pub(super) async fn handle_outgoing_msg(
    socket: &UdpSocket,
    mtu: usize,
    msg: crate::transport::OutboundMsg,
    sessions: &mut HashMap<SocketAddr, SessionState>,
) {
    let now = Instant::now();
    let state = sessions.entry(msg.peer).or_insert_with(|| {
        let (tx, rx) = mpsc::channel(128);
        let config = server_session_config();
        SessionState {
            managed: ManagedSession::with_config(msg.peer, mtu, now, config),
            to_app: tx,
            pending_rx: Some(rx),
            announced: false,
        }
    });

    let _ = state
        .managed
        .queue_app_packet(msg.packet, msg.reliability, msg.channel, msg.priority);

    tracing::trace!(
        peer = %msg.peer,
        connected = state.managed.is_connected(),
        "outbound queued"
    );
    flush_managed(&mut state.managed, socket, msg.peer, now).await;
}

pub(super) async fn tick_sessions(
    socket: &UdpSocket,
    sessions: &mut HashMap<SocketAddr, SessionState>,
) {
    let now = Instant::now();
    let mut dead = Vec::new();

    for (&peer, state) in sessions.iter_mut() {
        flush_managed(&mut state.managed, socket, peer, now).await;
        if matches!(state.managed.state(), ConnectionState::Closed) {
            // Inform app of disconnection if it was connected/announced
            if state.announced {
                if let Some(reason) = state.managed.last_disconnect_reason() {
                    let _ = state
                        .to_app
                        .send(Err(crate::RaknetError::Disconnected(reason)))
                        .await;
                } else {
                    let _ = state
                        .to_app
                        .send(Err(crate::RaknetError::ConnectionClosed))
                        .await;
                }
            }
            dead.push(peer);
        }
    }

    for peer in dead {
        sessions.remove(&peer);
    }
}

async fn handle_incoming_udp(
    socket: &UdpSocket,
    mtu: usize,
    bytes: &[u8],
    peer: SocketAddr,
    sessions: &mut HashMap<SocketAddr, SessionState>,
    _pending: &mut HashMap<SocketAddr, PendingConnection>,
    new_conn_tx: &mpsc::Sender<(
        SocketAddr,
        mpsc::Receiver<Result<Bytes, crate::RaknetError>>,
    )>,
) -> bool {
    let mut slice = bytes;
    let dgram = match Datagram::decode(&mut slice) {
        Ok(d) => d,
        Err(e) => {
            tracing::debug!(peer = %peer, error = ?e, "failed to decode datagram");
            return false;
        }
    };
    let now = Instant::now();
    let state = sessions.entry(peer).or_insert_with(|| {
        tracing::debug!(peer = %peer, mtu = mtu, "create_session");
        let (tx, rx) = mpsc::channel(128);
        let config = server_session_config();
        let sess = ManagedSession::with_config(peer, mtu, now, config);
        SessionState {
            managed: sess,
            to_app: tx,
            pending_rx: Some(rx),
            announced: false,
        }
    });

    let closed_after = if let Ok(pkts) = state.managed.handle_datagram(dgram, now) {
        if tracing::enabled!(tracing::Level::TRACE) {
            tracing::trace!(
                peer = %peer,
                connected = state.managed.is_connected(),
                count = pkts.len(),
                "handle_datagram"
            );
            for pkt in &pkts {
                tracing::trace!(peer = %peer, id = format_args!("0x{:02x}", pkt.id()), "pkt");
            }
        }
        for pkt in ManagedSession::filter_app_packets(pkts) {
            if let RaknetPacket::UserData { id, payload } = pkt {
                // Reassemble original app bytes as go-raknet does: id byte + payload bytes.
                let mut buf = bytes::BytesMut::with_capacity(1 + payload.len());
                buf.put_u8(id);
                buf.extend_from_slice(&payload);
                let _ = state.to_app.send(Ok(buf.freeze())).await;
            }
        }
        false
    } else {
        false
    };

    maybe_announce_connection(peer, state, new_conn_tx).await;
    flush_managed(&mut state.managed, socket, peer, now).await;

    if closed_after || matches!(state.managed.state(), ConnectionState::Closed) {
        if state.announced {
            if let Some(reason) = state.managed.last_disconnect_reason() {
                let _ = state
                    .to_app
                    .send(Err(crate::RaknetError::Disconnected(reason)))
                    .await;
            } else {
                let _ = state
                    .to_app
                    .send(Err(crate::RaknetError::ConnectionClosed))
                    .await;
            }
        }
        sessions.remove(&peer);
    }
    true
}

pub(super) async fn maybe_announce_connection(
    peer: SocketAddr,
    state: &mut SessionState,
    new_conn_tx: &mpsc::Sender<(
        SocketAddr,
        mpsc::Receiver<Result<Bytes, crate::RaknetError>>,
    )>,
) {
    if state.announced || !state.managed.is_connected() {
        tracing::trace!(
            peer = %peer,
            connected = state.managed.is_connected(),
            announced = state.announced,
            "maybe_announce"
        );
        return;
    }

    if let Some(rx) = state.pending_rx.take() {
        state.announced = true;
        tracing::info!(peer = %peer, "announce_connection");
        if new_conn_tx.send((peer, rx)).await.is_err() {
            state.announced = false;
        }
    }
}
