use serde_json::json;
use rumqttc::{AsyncClient, MqttOptions, QoS, Transport};
use tokio::sync::mpsc;
use tokio::time::Duration;
use std::collections::HashMap;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use std::sync::Arc;
use std::time::SystemTime;
use chrono;

use crate::config::Config;
use crate::encryption;
use crate::rate_limit::RateLimiter;
use crate::webhook::{WebhookManager, WebhookEvent};

// -----------------------------------------------------------------------------
// Message types exchanged between GUI and MQTT task
// -----------------------------------------------------------------------------
pub enum GuiToMqtt {
    SendMessage { channel: String, text: String },
    AddChannel { name: String, psk: String },
    RemoveChannel { name: String },
}

pub enum MqttToGui {
    ChannelMessageReceived { channel: String, text: String },
    NodeInfo { id: String, name: String },
    Position { id: String, lat: Option<f64>, lon: Option<f64>, alt: Option<f64> },
    Error(String),
    Info(String),   // Added for success messages (instead of abusing Error)
}

// -----------------------------------------------------------------------------
// Helper: validate channel name (no MQTT wildcards, 1-100 chars)
// -----------------------------------------------------------------------------
fn is_valid_channel_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && !name.contains('/')
        && !name.contains('#')
        && !name.contains('+')
}

// -----------------------------------------------------------------------------
// Helper: decrypt an incoming message if it starts with "ENC:"
// -----------------------------------------------------------------------------
fn decrypt_incoming_message(payload: &str, psk: Option<&str>) -> String {
    if let Some(ciphertext) = payload.strip_prefix("ENC:") {
        if let Some(psk) = psk {
            match encryption::decrypt_message(psk, ciphertext) {
                Ok(plaintext) => format!("🔒 {}", plaintext),
                Err(e) => {
                    eprintln!("Decryption failed: {}", e);
                    format!("[Encrypted - Decryption failed: {}]", e)
                }
            }
        } else {
            "[Encrypted - No PSK available]".to_string()
        }
    } else {
        payload.to_string()
    }
}

// -----------------------------------------------------------------------------
// Generate a unique node ID for this client (persists for the session)
// -----------------------------------------------------------------------------
fn generate_node_id() -> String {
    // Use timestamp + random to get a reasonably unique 8-character hex id
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let random = rand::random::<u32>();
    format!("{:08x}", (now as u32).wrapping_add(random))
}

// -----------------------------------------------------------------------------
// Main MQTT background task
// -----------------------------------------------------------------------------
pub async fn mqtt_task(
    mut cmd_rx: mpsc::UnboundedReceiver<GuiToMqtt>,
    gui_tx: mpsc::UnboundedSender<MqttToGui>,
    mut config: Config,
) -> Result<(), anyhow::Error> {
    // Validate initial channels from config, skip invalid ones
    let mut channels: HashMap<String, String> = HashMap::new();
    let initial_channels = std::mem::take(&mut config.initial_channels);
    for (name, psk) in initial_channels.into_iter() {
        if is_valid_channel_name(&name) {
            channels.insert(name, psk);
        } else {
            eprintln!("Skipping invalid initial channel: '{}'", name);
            let _ = gui_tx.send(MqttToGui::Error(format!(
                "Invalid initial channel name skipped: {}",
                name
            )));
        }
    }

    // Node ID – used in publish topics to avoid message loops
    let node_id = generate_node_id();

    let rate_limiter = Arc::new(RateLimiter::new());
    
    // Initialize webhook manager
    let webhook_manager = WebhookManager::new(config.webhook_configs.clone());
    let webhook_manager = Arc::new(webhook_manager);

    // MQTT root topic (configurable, default "msh/US/2/json")
    let root_topic = "msh/US/2/json".to_string();

    loop {
        let mut mqttoptions = MqttOptions::new("meshtastic-gui", &config.mqtt_host, config.mqtt_port);
        mqttoptions.set_keep_alive(Duration::from_secs(15));
        mqttoptions.set_credentials(&config.mqtt_username, &config.mqtt_password);

        // Configure TLS if enabled
        if config.mqtt_use_tls {
            let mut root_cert_store = RootCertStore::empty();

            // Try system certificates first
            match rustls_native_certs::load_native_certs() {
                Ok(certs) => {
                    root_cert_store.add_parsable_certificates(certs);
                    if root_cert_store.is_empty() {
                        eprintln!("Warning: no native root certificates found, falling back to webpki roots");
                        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                    }
                }
                Err(e) => {
                    eprintln!("Failed to load native certs: {}, falling back to webpki roots", e);
                    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                }
            }

            // If still empty, we cannot proceed with TLS
            if root_cert_store.is_empty() {
                eprintln!("No root certificates available for TLS – aborting TLS connection");
                let _ = gui_tx.send(MqttToGui::Error(
                    "TLS configured but no root certificates found".to_string(),
                ));
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }

            let client_config = ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();

            mqttoptions.set_transport(Transport::tls_with_config(client_config.into()));
        }

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Subscribe to all current channels
        for channel in channels.keys() {
            let topic = format!("{}/{}/#", root_topic, channel);
            if let Err(e) = client.subscribe(&topic, QoS::AtMostOnce).await {
                eprintln!("Subscribe failed for {}: {}", channel, e);
                let _ = gui_tx.send(MqttToGui::Error(format!("Subscribe failed: {}", e)));
            }
        }

        let client_sender = client.clone();

        'connection: loop {
            tokio::select! {
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {
                        GuiToMqtt::SendMessage { channel, text } => {
                            // Validate channel name
                            if !is_valid_channel_name(&channel) {
                                let _ = gui_tx.send(MqttToGui::Error("Invalid channel name".to_string()));
                                continue;
                            }

                            // Rate limiting
                            match rate_limiter.check_rate_limit(&channel).await {
                                Ok(_) => {}
                                Err(e) => {
                                    let _ = gui_tx.send(MqttToGui::Error(format!("Rate limit: {}", e)));
                                    continue;
                                }
                            }

                            // Sanitize input
                            if text.len() > 10000 {
                                let _ = gui_tx.send(MqttToGui::Error("Message too long (max 10000 chars)".to_string()));
                                continue;
                            }
                            let sanitized_text: String = text
                                .chars()
                                .filter(|c| *c != '\0' && (!c.is_control() || *c == '\n' || *c == '\t'))
                                .collect();

                            if let Some(psk) = channels.get(&channel) {
                                // Encrypt – fail hard if encryption fails (no plaintext fallback)
                                let payload = match encryption::encrypt_message(psk, &sanitized_text) {
                                    Ok(ciphertext) => format!("ENC:{}", ciphertext),
                                    Err(e) => {
                                        eprintln!("Encryption failed for channel {}: {}", channel, e);
                                        let _ = gui_tx.send(MqttToGui::Error(format!("Encryption failed: {}", e)));
                                        continue;
                                    }
                                };

                                let json_payload = json!({
                                    "type": "TEXT_MESSAGE_APP",
                                    "payload": payload,
                                    "channel": 0,
                                    "from": 0,
                                    "to": 0
                                });
                                let json_string = match serde_json::to_string(&json_payload) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        let _ = gui_tx.send(MqttToGui::Error(format!("JSON serialization error: {}", e)));
                                        continue;
                                    }
                                };
                                let send_topic = format!("{}/{}/{}", root_topic, channel, node_id);
                                if let Err(e) = client_sender
                                    .publish(&send_topic, QoS::AtLeastOnce, false, json_string.as_bytes())
                                    .await
                                {
                                    eprintln!("Send failed on {}: {}", channel, e);
                                    // Break on any send error to reconnect
                                    break 'connection;
                                }
                            } else {
                                let _ = gui_tx.send(MqttToGui::Error(format!("Unknown channel: {}", channel)));
                            }
                        }

                        GuiToMqtt::AddChannel { name, psk } => {
                            if !is_valid_channel_name(&name) {
                                let _ = gui_tx.send(MqttToGui::Error(
                                    "Invalid channel name. Cannot contain /, #, or + and must be 1-100 chars".to_string(),
                                ));
                                continue;
                            }
                            if psk.len() > 1000 {
                                let _ = gui_tx.send(MqttToGui::Error("PSK too long (max 1000 chars)".to_string()));
                                continue;
                            }
                            if !channels.contains_key(&name) {
                                channels.insert(name.clone(), psk);
                                let topic = format!("{}/{}/#", root_topic, name);
                                if let Err(e) = client_sender.subscribe(&topic, QoS::AtMostOnce).await {
                                    eprintln!("Failed to subscribe to {}: {}", name, e);
                                    let _ = gui_tx.send(MqttToGui::Error(format!("Subscribe failed: {}", e)));
                                } else {
                                    let _ = gui_tx.send(MqttToGui::Info(format!("Channel {} added", name)));
                                }
                            } else {
                                let _ = gui_tx.send(MqttToGui::Error(format!("Channel {} already exists", name)));
                            }
                        }

                        GuiToMqtt::RemoveChannel { name } => {
                            if channels.remove(&name).is_some() {
                                let topic = format!("{}/{}/#", root_topic, name);
                                if let Err(e) = client_sender.unsubscribe(&topic).await {
                                    eprintln!("Failed to unsubscribe from {}: {}", name, e);
                                    let _ = gui_tx.send(MqttToGui::Error(format!("Unsubscribe failed: {}", e)));
                                } else {
                                    let _ = gui_tx.send(MqttToGui::Info(format!("Channel {} removed", name)));
                                }
                            } else {
                                let _ = gui_tx.send(MqttToGui::Error(format!("Channel {} not found", name)));
                            }
                        }
                    }
                }

                result = eventloop.poll() => {
                    match result {
                        Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish))) => {
                            let parts: Vec<&str> = publish.topic.split('/').collect();
                            // Expected format: root/channel/...
                            if parts.len() >= 4 && parts[3] == "json" {
                                let channel = parts[4].to_string();
                                if let Ok(text) = String::from_utf8(publish.payload.to_vec()) {
                                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&text) {
                                        if let Some(msg_type) = json_val.get("type").and_then(|t| t.as_str()) {
                                            match msg_type {
                                                "TEXT_MESSAGE_APP" => {
                                                    let payload = json_val.get("payload")
                                                        .and_then(|p| p.as_str())
                                                        .unwrap_or(&text)
                                                        .to_string();
                                                    let psk = channels.get(&channel).map(String::as_str);
                                                    let decrypted = decrypt_incoming_message(&payload, psk);
                                                    
                                                    // Send webhook event for message received
                                                    let webhook_event = WebhookEvent::MessageReceived {
                                                        source: "mqtt".to_string(),
                                                        channel: Some(channel.clone()),
                                                        text: decrypted.clone(),
                                                        sender_id: json_val.get("from").and_then(|f| f.as_u64()).map(|f| f.to_string()),
                                                        timestamp: chrono::Utc::now(),
                                                    };
                                                    webhook_manager.send_event(&webhook_event).await;
                                                    
                                                    let _ = gui_tx.send(MqttToGui::ChannelMessageReceived {
                                                        channel,
                                                        text: decrypted,
                                                    });
                                                    continue;
                                                }
                                                "NODEINFO_APP" => {
                                                    let from = json_val.get("from")
                                                        .and_then(|f| f.as_u64())
                                                        .unwrap_or(0)
                                                        .to_string();
                                                    let name = json_val.get("payload")
                                                        .and_then(|p| p.as_str())
                                                        .unwrap_or("")
                                                        .to_string();
                                                    let _ = gui_tx.send(MqttToGui::NodeInfo { id: from, name });
                                                    continue;
                                                }
                                                "POSITION_APP" => {
                                                    let from = json_val.get("from")
                                                        .and_then(|f| f.as_u64())
                                                        .unwrap_or(0)
                                                        .to_string();
                                                    if let Some(payload_str) = json_val.get("payload").and_then(|p| p.as_str()) {
                                                        match serde_json::from_str::<serde_json::Value>(payload_str) {
                                                            Ok(pos_json) => {
                                                                let lat = pos_json.get("latitude").and_then(|v| v.as_f64());
                                                                let lon = pos_json.get("longitude").and_then(|v| v.as_f64());
                                                                let alt = pos_json.get("altitude").and_then(|v| v.as_f64());
                                                                let _ = gui_tx.send(MqttToGui::Position { id: from, lat, lon, alt });
                                                                continue;
                                                            }
                                                            Err(e) => {
                                                                eprintln!("Failed to parse position JSON for node {}: {}", from, e);
                                                                let _ = gui_tx.send(MqttToGui::Error(format!("Malformed position data: {}", e)));
                                                            }
                                                        }
                                                    } else {
                                                        eprintln!("Position message missing payload field");
                                                        let _ = gui_tx.send(MqttToGui::Error("Position missing payload".to_string()));
                                                    }
                                                    continue;
                                                }
                                                _ => {} // ignore other types
                                            }
                                        }
                                    }
                                    // Fallback: treat as plain text (with decryption support)
                                    let psk = channels.get(&channel).map(String::as_str);
                                    let decrypted = decrypt_incoming_message(&text, psk);
                                    
                                    // Send webhook event for message received (fallback case)
                                    let webhook_event = WebhookEvent::MessageReceived {
                                        source: "mqtt".to_string(),
                                        channel: Some(channel.clone()),
                                        text: decrypted.clone(),
                                        sender_id: None, // Unknown sender in fallback case
                                        timestamp: chrono::Utc::now(),
                                    };
                                    webhook_manager.send_event(&webhook_event).await;
                                    
                                    let _ = gui_tx.send(MqttToGui::ChannelMessageReceived {
                                        channel,
                                        text: decrypted,
                                    });
                                } else {
                                    let _ = gui_tx.send(MqttToGui::Error("Non-UTF8 data received".to_string()));
                                }
                            }
                        }
                        Ok(_) => {} // other events ignored
                        Err(e) => {
                            eprintln!("MQTT connection lost: {}, reconnecting...", e);
                            break 'connection;
                        }
                    }
                }
            }
        }

        // Wait before reconnecting to avoid rapid loops
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
}