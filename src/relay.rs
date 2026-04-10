use tokio::sync::mpsc;
use crate::mqtt::{GuiToMqtt, MqttToGui};
use crate::reticulum_bridge::{BridgeCommand, BridgeEvent};

pub struct RelayConfig {
    pub enabled: bool,
    pub direction: RelayDirection, // Both, MqttToReticulum, ReticulumToMqtt
}

#[derive(Clone, Copy, PartialEq)]
pub enum RelayDirection {
    Both,
    MqttToReticulum,
    ReticulumToMqtt,
}

impl Default for RelayDirection {
    fn default() -> Self { RelayDirection::Both }
}

/// Spawn a relay task that forwards messages between the two networks.
pub async fn spawn_relay(
    mut mqtt_rx: mpsc::UnboundedReceiver<MqttToGui>,
    mqtt_tx: mpsc::UnboundedSender<GuiToMqtt>,
    mut retic_rx: mpsc::UnboundedReceiver<BridgeEvent>,
    retic_tx: mpsc::UnboundedSender<BridgeCommand>,
    config: RelayConfig,
    target_channel: String,       // MQTT channel to forward to
    target_peer: Option<String>,  // Reticulum destination hash (if None, broadcast? but need a peer)
) {
    loop {
        tokio::select! {
            Some(msg) = mqtt_rx.recv() => {
                if !config.enabled { continue; }
                if matches!(config.direction, RelayDirection::Both | RelayDirection::MqttToReticulum) {
                    // Forward MQTT text message to Reticulum
                    if let MqttToGui::ChannelMessageReceived { text, .. } = msg {
                        if let Some(peer_hash) = &target_peer {
                            let _ = retic_tx.send(BridgeCommand::SendMessage {
                                dest_hash: peer_hash.clone(),
                                text,
                            });
                        }
                    }
                }
            }
            Some(event) = retic_rx.recv() => {
                if !config.enabled { continue; }
                if matches!(config.direction, RelayDirection::Both | RelayDirection::ReticulumToMqtt) {
                    if let BridgeEvent::MessageReceived { text, .. } = event {
                        let _ = mqtt_tx.send(GuiToMqtt::SendMessage {
                            channel: target_channel.clone(),
                            text,
                        });
                    }
                }
            }
        }
    }
}