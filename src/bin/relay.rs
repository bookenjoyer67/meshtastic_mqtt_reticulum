use meshtastic_reticulum_bridge::socket_bridge::spawn_socket_bridge;
use meshtastic_reticulum_bridge::reticulum_bridge::{BridgeCommand, BridgeEvent};
use meshtastic_reticulum_bridge::mqtt::{mqtt_task, GuiToMqtt, MqttToGui};
use meshtastic_reticulum_bridge::config::Config;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (mqtt_gui_tx, mut mqtt_gui_rx) = mpsc::unbounded_channel::<MqttToGui>();
    let (gui_mqtt_tx, gui_mqtt_rx) = mpsc::unbounded_channel::<GuiToMqtt>();
    let _mqtt_tx_clone = gui_mqtt_tx.clone();

    let config = Config::from_env();
    
    tokio::spawn(async move {
        if let Err(e) = mqtt_task(gui_mqtt_rx, mqtt_gui_tx, config).await {
            eprintln!("MQTT task error: {}", e);
        }
    });

    let (bridge_cmd_tx, bridge_cmd_rx) = mpsc::unbounded_channel::<BridgeCommand>();
    let (bridge_event_tx, mut bridge_event_rx) = mpsc::unbounded_channel::<BridgeEvent>();

    tokio::spawn(async move {
        if let Err(e) = spawn_socket_bridge(bridge_cmd_rx, bridge_event_tx).await {
            eprintln!("Socket bridge error: {}", e);
        }
    });

    let mut target_channel = "".to_string();
    let mut selected_peer: Option<String> = None;

    println!("Gateway started. Waiting for messages...");

    loop {
        tokio::select! {
            Some(msg) = mqtt_gui_rx.recv() => {
                if let MqttToGui::ChannelMessageReceived { channel, text } = msg {
                    if channel == target_channel {
                        if let Some(ref peer) = selected_peer {
                            let _ = bridge_cmd_tx.send(BridgeCommand::SendMessage {
                                dest_hash: peer.clone(),
                                text,
                            });
                        } else {
                            eprintln!("No Reticulum peer selected. Use !connect <hash> command from Reticulum.");
                        }
                    }
                }
            }
            Some(event) = bridge_event_rx.recv() => {
                match event {
                    BridgeEvent::MessageReceived { from, text } => {
                        if text.starts_with('!') {
                            let parts: Vec<&str> = text.split_whitespace().collect();
                            match parts[0] {
                                "!connect" => {
                                    if parts.len() > 1 {
                                        selected_peer = Some(parts[1].to_string());
                                        let _ = bridge_cmd_tx.send(BridgeCommand::SendMessage {
                                            dest_hash: from.clone(),
                                            text: "Connected to gateway".to_string(),
                                        });
                                    }
                                }
                                "!channel" => {
                                    if parts.len() > 1 {
                                        target_channel = parts[1].to_string();
                                        let _ = bridge_cmd_tx.send(BridgeCommand::SendMessage {
                                            dest_hash: from,
                                            text: format!("Switched to channel {}", target_channel),
                                        });
                                    }
                                }
                                _ => {}
                            }
                        } else {
                            let _ = gui_mqtt_tx.send(GuiToMqtt::SendMessage {
                                channel: target_channel.clone(),
                                text,
                            });
                        }
                    }
                    BridgeEvent::PeerDiscovered { main_hash, file_hash: _, last_seen: _, signal_strength: _, link_quality: _, interface: _ } => {
                        println!("Discovered Reticulum peer: {}", main_hash);
                    }
                    _ => {}
                }
            }
        }
    }
}