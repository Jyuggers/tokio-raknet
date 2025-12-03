use mimalloc::MiMalloc;
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::lookup_host;
use tokio_raknet::transport::{Message, RaknetListener, RaknetStream};
use tracing::Level;
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let fmt_layer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

    let filter_layer = filter::LevelFilter::from_level(Level::DEBUG);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();

    let bind_addr: SocketAddr = "0.0.0.0:19132".parse()?;
    let target_host = "play.lbsg.net:19132"; // play.lbsg.net

    tracing::info!("RakNet Forwarder starting...");
    tracing::info!("Listening on: {}", bind_addr);
    tracing::info!("Forwarding to: {}", target_host);

    let mut listener = RaknetListener::bind(bind_addr, 1200).await?;

    // Accept only one connection
    if let Some(client_stream) = listener.accept().await {
        let target = target_host.to_string();
        // Handle the connection in the main task (blocking)
        if let Err(e) = handle_connection(client_stream, target).await {
            tracing::error!("Connection error: {:?}", e);
        }
    }

    tracing::info!("Shutting down...");
    Ok(())
}

async fn handle_connection(
    mut client: RaknetStream,
    target_host: String,
) -> Result<(), Box<dyn Error>> {
    let client_addr = client.peer_addr();
    tracing::info!("[{}] New client connected", client_addr);

    // Resolve target
    tracing::info!("[{}] Resolving {}...", client_addr, target_host);
    let mut addrs = lookup_host(&target_host).await?;
    let remote_addr = addrs.next().ok_or("Failed to resolve target host")?;

    tracing::info!("[{}] Connecting to server {}...", client_addr, remote_addr);
    let mut server = RaknetStream::connect(remote_addr, 1200).await?;
    tracing::info!("[{}] Connected to server!", client_addr);

    // Split the streams locally to manage concurrent read/writes
    // Since RaknetStream doesn't support 'split()' yet or Clone, we have to wrap in Arc<Mutex>
    // OR just use a select loop with &mut.
    // Using select! loop is cleaner than Arc<Mutex> for this case.

    loop {
        tokio::select! {
            // Client -> Server
            res = client.recv_msg() => {
                match res {
                    Some(Ok(packet)) => {
                        let outbound = Message::new(packet.buffer)
                            .reliability(packet.reliability)
                            .channel(packet.channel);
                        server.send(outbound).await?;
                    }
                    Some(Err(e)) => {
                        tracing::info!("[{}] Client error: {:?}", client_addr, e);
                        break;
                    }
                    None => {
                        tracing::info!("[{}] Client disconnected", client_addr);
                        break;
                    }
                }
            }

            // Server -> Client
            res = server.recv_msg() => {
                match res {
                    Some(Ok(packet)) => {
                        let outbound = Message::new(packet.buffer)
                            .reliability(packet.reliability)
                            .channel(packet.channel);
                        client.send(outbound).await?;
                    }
                    Some(Err(e)) => {
                        tracing::info!("[{}] Server error: {:?}", client_addr, e);
                        break;
                    }
                    None => {
                        tracing::info!("[{}] Server disconnected", client_addr);
                        break;
                    }
                }
            }
        }
    }

    tracing::info!("[{}] Closing connection...", client_addr);
    // Send disconnect packet (0x15) to both ends just in case
    // This ensures the client gets a clean disconnect even if the Listener doesn't detect the drop immediately.
    let disconnect_msg = Message::new(vec![0x15]);
    let _ = client.send(disconnect_msg.clone()).await;
    let _ = server.send(disconnect_msg).await;

    tracing::info!("[{}] Connection closed", client_addr);
    Ok(())
}
