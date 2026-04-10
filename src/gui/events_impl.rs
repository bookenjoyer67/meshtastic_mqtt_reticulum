use eframe::egui;
use chrono::Local;
use crate::gui::{MqttToGui, GuiToMqtt};
use crate::gui::{BridgeEvent, BridgeCommand};
use crate::gui::app::MeshtasticGuiApp;
use crate::gui::peers::Peer;
use crate::gui::nodes::NodeInfo;
use crate::gui::relay::RelayDirection;

impl MeshtasticGuiApp {
    pub fn process_mqtt_events(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.mqtt_msg_rx.try_recv() {
            match msg {
                MqttToGui::ChannelMessageReceived { channel, text } => {
                    self.messages.push((channel, text.clone()));
                    if self.messages.len() > 200 { self.messages.remove(0); }
                    ctx.request_repaint();

                    if self.relay_enabled {
                        let direction_ok = matches!(self.relay_direction, RelayDirection::Both | RelayDirection::MqttToReticulum);
                        if direction_ok {
                            if let Some(peer) = &self.selected_peer {
                                let _ = self.bridge_cmd_tx.send(BridgeCommand::SendMessage {
                                    dest_hash: peer.main_hash.clone(),
                                    text,
                                });
                            }
                        }
                    }
                }
                MqttToGui::NodeInfo { id, name } => {
                    let now = Local::now();
                    if let Some(node) = self.nodes.get_mut(&id) {
                        node.name = name;
                        node.last_seen = now;
                    } else {
                        self.nodes.insert(id.clone(), NodeInfo {
                            id: id.clone(),
                            name,
                            latitude: None,
                            longitude: None,
                            altitude: None,
                            last_seen: now,
                        });
                    }
                    ctx.request_repaint();
                }
                MqttToGui::Position { id, lat, lon, alt } => {
                    let now = Local::now();
                    if let Some(node) = self.nodes.get_mut(&id) {
                        node.latitude = lat;
                        node.longitude = lon;
                        node.altitude = alt.map(|a| a as f32);
                        node.last_seen = now;
                    } else {
                        self.nodes.insert(id.clone(), NodeInfo {
                            id: id.clone(),
                            name: "Unknown".to_string(),
                            latitude: lat,
                            longitude: lon,
                            altitude: alt.map(|a| a as f32),
                            last_seen: now,
                        });
                    }
                    ctx.request_repaint();
                }
                MqttToGui::Error(err) => {
                    self.messages.push(("MQTT".to_string(), err));
                    ctx.request_repaint();
                }
                MqttToGui::Info(info) => {
                    self.messages.push(("MQTT".to_string(), format!("Info: {}", info)));
                    ctx.request_repaint();
                }
            }
        }
    }

    pub fn process_reticulum_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.bridge_msg_rx.try_recv() {
            match event {
                BridgeEvent::MessageReceived { from, text } => {
                    let (display_nick, display_text) = if text.starts_with('[') {
                        if let Some(closing) = text.find(']') {
                            let nick = &text[1..closing];
                            let rest = &text[closing+1..].trim_start();
                            (nick.to_string(), rest.to_string())
                        } else {
                            ("Unknown".to_string(), text.clone())
                        }
                    } else {
                        ("Unknown".to_string(), text.clone())
                    };
                    if display_nick != *"Unknown" {
                        let current = self.peer_nicknames.get(&from).cloned().unwrap_or_default();
                        if current != display_nick {
                            self.set_peer_nickname(&from, &display_nick, ctx);
                        }
                    }
                    self.messages.push((
                        "Reticulum".to_string(),
                        format!("[{}] {}: {}", display_nick, from, display_text)
                    ));
                    ctx.request_repaint();

                    if self.relay_enabled {
                        let direction_ok = matches!(self.relay_direction, RelayDirection::Both | RelayDirection::ReticulumToMqtt);
                        if direction_ok {
                            let _ = self.mqtt_cmd_tx.send(GuiToMqtt::SendMessage {
                                channel: self.relay_target_channel.clone(),
                                text: text.clone(),
                            });
                        }
                    }
                }
                BridgeEvent::PeerDiscovered { main_hash, file_hash, last_seen, signal_strength, link_quality, interface } => {
    let clean_main = main_hash.trim_matches('/').to_string();
    let clean_file = file_hash.trim_matches('/').to_string();
    if !self.peers.iter().any(|p| p.main_hash == clean_main) {
        // Parse last_seen timestamp if available
        let last_seen_dt = last_seen.and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&chrono::Local)));
        
        // Create new peer with additional metadata
        let peer = Peer {
            main_hash: clean_main.clone(),
            file_hash: clean_file.clone(),
            name: None, // Will be set from nicknames if available
            last_seen: last_seen_dt,
            signal_strength,
            link_quality,
            interface,
            network_metrics: None,
            radio_metrics: None,
            last_metrics_update: None,
        };
        
        self.peers.push(peer);
        self.save_peer(&clean_main, &clean_file);
        ctx.request_repaint();
    } else {
        // Update existing peer with new metadata
        if let Some(existing_peer) = self.peers.iter_mut().find(|p| p.main_hash == clean_main) {
            // Update last_seen if provided
            if let Some(ls) = last_seen {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&ls) {
                    existing_peer.last_seen = Some(dt.with_timezone(&chrono::Local));
                }
            }
            // Update signal strength if provided
            if signal_strength.is_some() {
                existing_peer.signal_strength = signal_strength;
            }
            // Update link quality if provided
            if link_quality.is_some() {
                existing_peer.link_quality = link_quality;
            }
            // Update interface if provided
            if interface.is_some() {
                existing_peer.interface = interface;
            }
            ctx.request_repaint();
        }
    }
}
                BridgeEvent::FileTransferProgress { file_name, bytes_sent, total_bytes } => {
                    let progress = bytes_sent as f32 / total_bytes as f32;
                    self.transfer_progress = Some((file_name, progress));
                    ctx.request_repaint();
                }
                BridgeEvent::FileTransferComplete { file_name } => {
                    self.messages.push(("Reticulum".to_string(), format!("File {} sent", file_name)));
                    self.transfer_progress = None;
                    ctx.request_repaint();
                }
                BridgeEvent::FileTransferError { file_name, error } => {
                    self.messages.push(("Reticulum".to_string(), format!("File transfer error for {}: {}", file_name, error)));
                    self.transfer_progress = None;
                    ctx.request_repaint();
                }
                BridgeEvent::FileReceived { file_name, file_path } => {
                    self.messages.push(("Reticulum".to_string(), format!("File {} received at {}", file_name, file_path)));
                    ctx.request_repaint();
                }
                BridgeEvent::Error(err) => {
                    self.messages.push(("Reticulum".to_string(), format!("Error: {}", err)));
                    ctx.request_repaint();
                }
                BridgeEvent::InterfaceStatus { name, connected, bytes_sent, bytes_received, error } => {
                    if let Some(status) = self.interface_statuses.get_mut(&name) {
                        status.connected = connected;
                        status.bytes_sent = bytes_sent;
                        status.bytes_received = bytes_received;
                        status.error = error.clone();
                        status.last_seen = Some(std::time::Instant::now());
                        
                        if connected {
                            self.messages.push(("System".to_string(), format!("Interface {} connected", name)));
                        } else if let Some(err) = error {
                            self.messages.push(("System".to_string(), format!("Interface {} error: {}", name, err)));
                        }
                    }
                    ctx.request_repaint();
                }
            }
        }
    }
}