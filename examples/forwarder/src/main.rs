use std::net::SocketAddr;
use std::{error::Error, fs::OpenOptions};
use tokio::net::lookup_host;
use tokio_raknet::transport::{Message, RaknetListener, RaknetStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let log_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("application.log")
        .unwrap();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(log_file)
        .init();

    let bind_addr: SocketAddr = "0.0.0.0:19132".parse()?;
    let target_host = "test.oomph.ac:19132"; // play.lbsg.net

    println!("RakNet Forwarder starting...");
    println!("Listening on: {}", bind_addr);
    println!("Forwarding to: {}", target_host);

    let mut listener = RaknetListener::bind(bind_addr, 1400).await?;

    loop {
        tokio::select! {
            conn = listener.accept() => {
                match conn {
                    Some(client_stream) => {
                        let target = target_host.to_string();
                        tokio::spawn(async move {
                            if let Err(e) = handle_connection(client_stream, target).await {
                                eprintln!("Connection error: {:?}", e);
                            }
                        });
                    }
                    None => break,
                }
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Shutting down...");
                break;
            }
        }
    }

    Ok(())
}

async fn handle_connection(
    mut client: RaknetStream,
    target_host: String,
) -> Result<(), Box<dyn Error>> {
    let client_addr = client.peer_addr();
    println!("[{}] New client connected", client_addr);

    // Resolve target
    println!("[{}] Resolving {}...", client_addr, target_host);
    let mut addrs = lookup_host(&target_host).await?;
    let remote_addr = addrs.next().ok_or("Failed to resolve target host")?;

    println!("[{}] Connecting to server {}...", client_addr, remote_addr);
    let mut server = RaknetStream::connect(remote_addr, 1200).await?;
    println!("[{}] Connected to server!", client_addr);

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
                        println!("[{}] Client error: {:?}", client_addr, e);
                        break;
                    }
                    None => {
                        println!("[{}] Client disconnected", client_addr);
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
                        println!("[{}] Server error: {:?}", client_addr, e);
                        break;
                    }
                    None => {
                        println!("[{}] Server disconnected", client_addr);
                        break;
                    }
                }
            }
        }
    }

    println!("[{}] Connection closed", client_addr);
    Ok(())
}
