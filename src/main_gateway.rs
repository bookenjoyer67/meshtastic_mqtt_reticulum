use meshtastic_reticulum_bridge::socket_bridge::{spawn_socket_bridge, Peer};
use meshtastic_reticulum_bridge::reticulum_bridge::{BridgeCommand, BridgeEvent};
use meshtastic_reticulum_bridge::mqtt::{mqtt_task, GuiToMqtt, MqttToGui};
use meshtastic_reticulum_bridge::config::Config;
use tokio::sync::mpsc;
use chrono;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Channels for MQTT
    let (mqtt_gui_tx, mut mqtt_gui_rx) = mpsc::unbounded_channel::<MqttToGui>();
    let (gui_mqtt_tx, gui_mqtt_rx) = mpsc::unbounded_channel::<GuiToMqtt>();
    let _mqtt_tx_clone = gui_mqtt_tx.clone();

    // Spawn MQTT task (same as GUI version)
    let config = Config::from_env();
    tokio::spawn(async move {
        if let Err(e) = mqtt_task(gui_mqtt_rx, mqtt_gui_tx, config).await {
            eprintln!("MQTT task error: {}", e);
        }
    });

    // Channels for Reticulum bridge
    let (bridge_cmd_tx, bridge_cmd_rx) = mpsc::unbounded_channel::<BridgeCommand>();
    let (bridge_event_tx, mut bridge_event_rx) = mpsc::unbounded_channel::<BridgeEvent>();

    // Spawn bridge (TCP connection to the Rust Reticulum bridge)
    tokio::spawn(async move {
        if let Err(e) = spawn_socket_bridge(bridge_cmd_rx, bridge_event_tx).await {
            eprintln!("Socket bridge error: {}", e);
        }
    });

    // Configuration: which Meshtastic channel to forward to
    let mut target_channel = "".to_string();

    // Relay loop: forward Reticulum messages to MQTT, and MQTT messages to Reticulum
    // For Reticulum, we need a default destination hash. For simplicity, we'll forward to all peers? Actually we need to select a peer.
    // But as a gateway, we can maintain a list of peers and forward to a configured one.
    // Let's assume the gateway has a fixed Reticulum peer (the user) – but that's not automated.
    // Better: the gateway listens for commands from Reticulum to set the destination.
    // We'll store a list of discovered Reticulum peers (from BridgeEvent::PeerDiscovered) and allow a user to select one via command.

    let mut retic_peers: Vec<Peer> = Vec::new();
    let mut selected_peer: Option<Peer> = None;

    loop {
        tokio::select! {
            // Handle incoming MQTT messages (from Meshtastic)
            Some(msg) = mqtt_gui_rx.recv() => {
                if let MqttToGui::ChannelMessageReceived { channel, text } = msg {
                    if channel == target_channel {
                        // Forward to Reticulum (to the selected peer, or broadcast if none)
                        if let Some(peer) = &selected_peer {
                            let _ = bridge_cmd_tx.send(BridgeCommand::SendMessage {
                                dest_hash: peer.main_hash.clone(),
                                text,
                            });
                        } else {
                            eprintln!("No Reticulum peer selected. Use !connect <hash>");
                        }
                    }
                }
            }
            // Handle incoming Reticulum messages
            Some(event) = bridge_event_rx.recv() => {
                match event {
                    BridgeEvent::MessageReceived { from, text } => {
                        // Check if it's a command
                        if text.starts_with('!') {
                            let parts: Vec<&str> = text.split_whitespace().collect();
                            match parts[0] {
                                "!channel" => {
                                    if parts.len() > 1 {
                                        target_channel = parts[1].to_string();
                                        println!("Switched to Meshtastic channel: {}", target_channel);
                                    }
                                }
                                "!connect" => {
                                    if parts.len() > 1 {
                                        let hash = parts[1];
                                        if let Some(peer) = retic_peers.iter().find(|p| p.main_hash == hash) {
                                            selected_peer = Some(peer.clone());
                                            println!("Selected Reticulum peer: {}", hash);
                                        } else {
                                            println!("Peer not found: {}", hash);
                                        }
                                    }
                                }
                                "!peers" => {
                                    let list = retic_peers.iter().map(|p| p.main_hash.clone()).collect::<Vec<_>>().join(", ");
                                    let _ = bridge_cmd_tx.send(BridgeCommand::SendMessage {
                                        dest_hash: from.clone(),
                                        text: format!("Peers: {}", list),
                                    });
                                }
                                _ => {
                                    // Unknown command, ignore or reply
                                }
                            }
                        } else {
                            // Regular text: forward to Meshtastic channel
                            let _ = gui_mqtt_tx.send(GuiToMqtt::SendMessage {
                                channel: target_channel.clone(),
                                text,
                            });
                        }
                    }
                    BridgeEvent::PeerDiscovered { main_hash, file_hash, last_seen, signal_strength, link_quality, interface } => {
                        let peer = Peer { 
                            main_hash: main_hash.clone(), 
                            file_hash: file_hash.clone(),
                            name: None,
                            last_seen: last_seen.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Local))),
                            signal_strength,
                            link_quality,
                            interface,
                        };
                        if !retic_peers.iter().any(|p| p.main_hash == peer.main_hash) {
                            retic_peers.push(peer);
                            println!("Discovered Reticulum peer: {}", main_hash);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}