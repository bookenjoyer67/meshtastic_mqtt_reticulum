use reticulum::iface::tcp_client::TcpClient;
use reticulum::transport::{Transport, TransportConfig};
use reticulum::destination::DestinationName;
use reticulum::identity::PrivateIdentity;
use rand_core::OsRng;
use tokio::net::TcpListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::Mutex;
use log::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // ---------- Reticulum setup ----------
    let mut transport = Transport::new(TransportConfig::default());
    
    // Get Reticulum server address from environment variable with fallback
    let reticulum_server = std::env::var("RETICULUM_SERVER")
        .unwrap_or_else(|_| "RNS.MichMesh.net:7822".to_string());
    
    transport
        .iface_manager()
        .lock()
        .await
        .spawn(
            TcpClient::new(&reticulum_server),
            TcpClient::spawn,
        );
    info!("Transport created.");

    let identity = PrivateIdentity::new_from_rand(OsRng);
    let dest_name = DestinationName::new("meshtastic_bridge", "app");
    let destination = transport.add_destination(identity, dest_name).await;
    let my_hash = destination.lock().await.desc.address_hash.clone();
    info!("My hash: {}", my_hash);

    let announce = destination.lock().await.announce(OsRng, None)?;
    transport.send_packet(announce).await;
    info!("Announced.");

    let transport = Arc::new(Mutex::new(transport));
    let _destination = Arc::new(Mutex::new(destination));

    // ---------- Discover peers ----------
    let (peer_tx, mut peer_rx) = mpsc::unbounded_channel::<String>();
    let transport_discovery = transport.clone();
    tokio::spawn(async move {
        let mut announce_receiver = transport_discovery.lock().await.recv_announces().await;
        while let Ok(announce) = announce_receiver.recv().await {
            let remote_hash = announce.destination.lock().await.desc.address_hash.clone();
            if remote_hash != my_hash {
                let _ = peer_tx.send(remote_hash.to_string());
            }
        }
    });

    // ---------- TCP server for GUI ----------
    let listener = TcpListener::bind("127.0.0.1:4243").await?;
    info!("Listening for GUI on port 4243");
    let (socket, _) = listener.accept().await?;
    let (reader, writer) = socket.into_split();
    let reader = BufReader::new(reader);
    info!("GUI connected");

    // Forward discovered peers to GUI
    let (gui_tx, mut gui_rx) = mpsc::unbounded_channel::<String>();
    let mut gui_writer = writer;
    tokio::spawn(async move {
        while let Some(msg) = gui_rx.recv().await {
            let _ = gui_writer.write_all(msg.as_bytes()).await;
        }
    });

    // Send discovered peers to GUI
    let gui_tx_peers = gui_tx.clone();
    tokio::spawn(async move {
        while let Some(peer_hash) = peer_rx.recv().await {
            let msg = format!("{{\"type\": \"announce\", \"main_hash\": \"{}\", \"file_hash\": \"{}\"}}\n", peer_hash, peer_hash);
            let _ = gui_tx_peers.send(msg);
        }
    });

    // Handle commands from GUI
    let _transport_cmd = transport.clone();
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        info!("Received from GUI: {}", line);
        // Simple echo for now - in a real implementation, parse JSON and handle commands
        let response = format!("{{\"type\": \"echo\", \"message\": \"{}\"}}\n", line);
        let _ = gui_tx.send(response);
    }

    Ok(())
}