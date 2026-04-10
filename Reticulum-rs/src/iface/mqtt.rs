use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::buffer::{InputBuffer, OutputBuffer};
use crate::iface::RxMessage;
use crate::packet::Packet;
use crate::serde::Serialize;

use super::hdlc::Hdlc;
use super::{Interface, InterfaceContext};

// TODO: Configure via features
const PACKET_TRACE: bool = false;

#[cfg(feature = "mqtt")]
use rumqttc::{AsyncClient, MqttOptions, QoS, Transport, Event, Incoming};
#[cfg(feature = "mqtt")]
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
#[cfg(feature = "mqtt")]
use tokio::time::Duration;

pub struct MqttInterface {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    use_tls: bool,
    client_id: String,
    topic_prefix: String,
}

impl MqttInterface {
    pub fn new<T: Into<String>>(host: T, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            username: None,
            password: None,
            use_tls: false,
            client_id: "reticulum-mqtt".to_string(),
            topic_prefix: "reticulum".to_string(),
        }
    }

    pub fn with_credentials<T: Into<String>>(mut self, username: T, password: T) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    pub fn with_tls(mut self, use_tls: bool) -> Self {
        self.use_tls = use_tls;
        self
    }

    pub fn with_client_id<T: Into<String>>(mut self, client_id: T) -> Self {
        self.client_id = client_id.into();
        self
    }

    pub fn with_topic_prefix<T: Into<String>>(mut self, topic_prefix: T) -> Self {
        self.topic_prefix = topic_prefix.into();
        self
    }

    #[cfg(feature = "mqtt")]
    pub async fn spawn(context: InterfaceContext<MqttInterface>) {
        let iface_stop = context.channel.stop.clone();
        let host = { context.inner.lock().unwrap().host.clone() };
        let port = { context.inner.lock().unwrap().port };
        let username = { context.inner.lock().unwrap().username.clone() };
        let password = { context.inner.lock().unwrap().password.clone() };
        let use_tls = { context.inner.lock().unwrap().use_tls };
        let client_id = { context.inner.lock().unwrap().client_id.clone() };
        let topic_prefix = { context.inner.lock().unwrap().topic_prefix.clone() };
        let iface_address = context.channel.address;

        let (rx_channel, tx_channel) = context.channel.split();
        let tx_channel = Arc::new(tokio::sync::Mutex::new(tx_channel));

        let running = true;
        loop {
            if !running || context.cancel.is_cancelled() {
                break;
            }

            // Create MQTT options
            let mut mqttoptions = MqttOptions::new(&client_id, &host, port);
            mqttoptions.set_keep_alive(Duration::from_secs(15));
            
            if let (Some(username), Some(password)) = (&username, &password) {
                mqttoptions.set_credentials(username, password);
            }

            // Configure TLS if enabled
            if use_tls {
                let mut root_cert_store = RootCertStore::empty();

                // Try system certificates first
                match rustls_native_certs::load_native_certs() {
                    Ok(certs) => {
                        root_cert_store.add_parsable_certificates(certs);
                        if root_cert_store.is_empty() {
                            log::warn!("Warning: no native root certificates found, falling back to webpki roots");
                            root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to load native certs: {}, falling back to webpki roots", e);
                        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                    }
                }

                // If still empty, we cannot proceed with TLS
                if root_cert_store.is_empty() {
                    log::error!("No root certificates available for TLS – aborting TLS connection");
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    continue;
                }

                let client_config = ClientConfig::builder()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth();

                mqttoptions.set_transport(Transport::tls_with_config(client_config.into()));
            }

            let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

            // Subscribe to reticulum topic
            let subscribe_topic = format!("{}/#", topic_prefix);
            if let Err(e) = client.subscribe(&subscribe_topic, QoS::AtMostOnce).await {
                log::error!("MQTT subscribe failed for {}: {}", subscribe_topic, e);
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }

            log::info!("mqtt_interface connected to <{}:{}>", host, port);

            const BUFFER_SIZE: usize = core::mem::size_of::<Packet>() * 2;

            // Start receive task
            let rx_task = {
                let cancel = context.cancel.clone();
                let stop = CancellationToken::new();
                let _client = client.clone();
                let rx_channel = rx_channel.clone();
                let topic_prefix = topic_prefix.clone();

                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = cancel.cancelled() => {
                                break;
                            }
                            _ = stop.cancelled() => {
                                break;
                            }
                            result = eventloop.poll() => {
                                match result {
                                    Ok(Event::Incoming(Incoming::Publish(publish))) => {
                                        // Check if topic starts with our prefix
                                        if publish.topic.starts_with(&topic_prefix) {
                                            // Extract data from payload
                                            let payload = publish.payload;
                                            
                                            // MQTT payload should contain HDLC-encoded Reticulum packets
                                            let mut hdlc_rx_buffer = [0u8; BUFFER_SIZE];
                                            let mut rx_buffer = [0u8; BUFFER_SIZE + (BUFFER_SIZE / 2)];
                                            
                                            // Copy payload to rx_buffer (starting from the end)
                                            let payload_len = payload.len();
                                            if payload_len > 0 && payload_len <= rx_buffer.len() {
                                                // Push payload bytes into buffer
                                                for (i, &byte) in payload.iter().enumerate() {
                                                    if i < rx_buffer.len() {
                                                        rx_buffer[rx_buffer.len() - payload_len + i] = byte;
                                                    }
                                                }
                                                
                                                // Check if it contains a HDLC frame
                                                let frame = Hdlc::find(&rx_buffer[..]);
                                                if let Some(frame) = frame {
                                                    // Decode HDLC frame and deserialize packet
                                                    let frame_buffer = &mut rx_buffer[frame.0..frame.1+1];
                                                    let mut output = OutputBuffer::new(&mut hdlc_rx_buffer[..]);
                                                    if let Ok(_) = Hdlc::decode(frame_buffer, &mut output) {
                                                        if let Ok(packet) = Packet::deserialize(&mut InputBuffer::new(output.as_slice())) {
                                                            if PACKET_TRACE {
                                                                log::trace!("mqtt_interface: rx << ({}) {}", iface_address, packet);
                                                            }
                                                            let _ = rx_channel.send(RxMessage { address: iface_address, packet }).await;
                                                        } else {
                                                            log::warn!("mqtt_interface: couldn't decode packet");
                                                        }
                                                    } else {
                                                        log::warn!("mqtt_interface: couldn't decode hdlc frame");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Ok(_) => {} // other events ignored
                                    Err(e) => {
                                        log::warn!("mqtt_interface: connection error {}", e);
                                        stop.cancel();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                })
            };

            // Start transmit task
            let tx_task = {
                let cancel = context.cancel.clone();
                let tx_channel = tx_channel.clone();
                let client = client.clone();
                let topic_prefix = topic_prefix.clone();

                tokio::spawn(async move {
                    loop {
                        let mut hdlc_tx_buffer = [0u8; BUFFER_SIZE];
                        let mut tx_buffer = [0u8; BUFFER_SIZE];

                        let mut tx_channel = tx_channel.lock().await;

                        tokio::select! {
                            _ = cancel.cancelled() => {
                                break;
                            }
                            Some(message) = tx_channel.recv() => {
                                let packet = message.packet;
                                if PACKET_TRACE {
                                    log::trace!("mqtt_interface: tx >> ({}) {}", iface_address, packet);
                                }
                                let mut output = OutputBuffer::new(&mut tx_buffer);
                                if let Ok(_) = packet.serialize(&mut output) {
                                    let mut hdlc_output = OutputBuffer::new(&mut hdlc_tx_buffer[..]);

                                    if let Ok(_) = Hdlc::encode(output.as_slice(), &mut hdlc_output) {
                                        // Publish to MQTT topic
                                        let publish_topic = format!("{}/broadcast", topic_prefix);
                                        if let Err(e) = client.publish(
                                            &publish_topic,
                                            QoS::AtLeastOnce,
                                            false,
                                            hdlc_output.as_slice()
                                        ).await {
                                            log::warn!("mqtt_interface: publish error {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                })
            };

            tx_task.await.unwrap();
            rx_task.await.unwrap();

            log::info!("mqtt_interface: disconnected from <{}:{}>", host, port);
        }

        iface_stop.cancel();
    }

    #[cfg(not(feature = "mqtt"))]
    pub async fn spawn(_context: InterfaceContext<MqttInterface>) {
        log::error!("mqtt_interface: mqtt feature not enabled");
    }
}

impl Interface for MqttInterface {
    fn mtu() -> usize {
        2048
    }
}