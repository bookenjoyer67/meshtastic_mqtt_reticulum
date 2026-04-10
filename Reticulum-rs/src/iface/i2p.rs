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

#[cfg(feature = "i2p")]
use i2p_client::{Session, SessionStyle};
#[cfg(feature = "i2p")]
use tokio::time::Duration;
#[cfg(feature = "i2p")]
use std::sync::Mutex;

pub struct I2PInterface {
    sam_address: String,
    sam_port: u16,
    destination: Option<String>,
    session_name: String,
    session_type: String,
    use_local_dest: bool,
    max_connection_attempts: u8,
}

impl I2PInterface {
    pub fn new<T: Into<String>>(sam_address: T, sam_port: u16) -> Self {
        Self {
            sam_address: sam_address.into(),
            sam_port,
            destination: None,
            session_name: "reticulum-i2p".to_string(),
            session_type: "STREAM".to_string(),
            use_local_dest: true,
            max_connection_attempts: 10,
        }
    }

    pub fn with_destination<T: Into<String>>(mut self, destination: T) -> Self {
        self.destination = Some(destination.into());
        self
    }

    pub fn with_session_name<T: Into<String>>(mut self, session_name: T) -> Self {
        self.session_name = session_name.into();
        self
    }

    pub fn with_session_type<T: Into<String>>(mut self, session_type: T) -> Self {
        self.session_type = session_type.into();
        self
    }

    pub fn with_use_local_dest(mut self, use_local_dest: bool) -> Self {
        self.use_local_dest = use_local_dest;
        self
    }

    pub fn with_max_connection_attempts(mut self, max_connection_attempts: u8) -> Self {
        self.max_connection_attempts = max_connection_attempts;
        self
    }

    #[cfg(feature = "i2p")]
    pub async fn spawn(context: InterfaceContext<I2PInterface>) {
        let iface_stop = context.channel.stop.clone();
        let sam_address = { context.inner.lock().unwrap().sam_address.clone() };
        let sam_port = { context.inner.lock().unwrap().sam_port };
        let destination = { context.inner.lock().unwrap().destination.clone() };
        let session_name = { context.inner.lock().unwrap().session_name.clone() };
        let session_type = { context.inner.lock().unwrap().session_type.clone() };
        let _use_local_dest = { context.inner.lock().unwrap().use_local_dest };
        let _max_connection_attempts = { context.inner.lock().unwrap().max_connection_attempts };
        let iface_address = context.channel.address;

        let (rx_channel, tx_channel) = context.channel.split();
        let tx_channel = Arc::new(tokio::sync::Mutex::new(tx_channel));

        let running = true;
        loop {
            if !running || context.cancel.is_cancelled() {
                break;
            }

            log::info!("i2p_interface: connecting to SAM bridge at {}:{}", sam_address, sam_port);

            // Create I2P session
            let sam_addr = format!("{}:{}", sam_address, sam_port);
            let session_style = match session_type.as_str() {
                "DATAGRAM" => SessionStyle::Datagram,
                "RAW" => SessionStyle::Raw,
                "STREAM" => SessionStyle::Stream,
                _ => {
                    log::warn!("i2p_interface: Unknown session type '{}', defaulting to STREAM", session_type);
                    SessionStyle::Stream
                }
            };

            let session = match Session::create(
                &sam_addr,
                "TRANSIENT",
                &session_name,
                session_style,
                "3.0",
                "3.1",
            ) {
                Ok(session) => {
                    log::info!("i2p_interface: I2P session created successfully");
                    Arc::new(Mutex::new(session))
                }
                Err(e) => {
                    log::error!("i2p_interface: failed to create I2P session: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            log::info!("i2p_interface: connected to SAM bridge at {}:{}", sam_address, sam_port);
            log::info!("i2p_interface: Session: {} (type: {})", session_name, session_type);
            
            if let Some(dest) = &destination {
                log::info!("i2p_interface: Destination: {}", dest);
            } else {
                log::info!("i2p_interface: No destination specified - will accept incoming connections");
            }

            const BUFFER_SIZE: usize = core::mem::size_of::<Packet>() * 2;

            // Start receive task
            let rx_task = {
                let cancel = context.cancel.clone();
                let stop = CancellationToken::new();
                let rx_channel = rx_channel.clone();
                let destination = destination.clone();
                let session = session.clone();

                tokio::spawn(async move {
                    loop {
                        tokio::select! {
                            _ = cancel.cancelled() => {
                                break;
                            }
                            _ = stop.cancelled() => {
                                break;
                            }
                            result = async {
                                // Receive message from I2P network (blocking call)
                                let receive_result = tokio::task::spawn_blocking({
                                    let session = session.clone();
                                    move || {
                                        let mut session_guard = session.lock().unwrap();
                                        session_guard.recv_msg()
                                    }
                                }).await;

                                match receive_result {
                                    Ok(Ok((from, data))) => {
                                        // Check if we have a destination filter
                                        if let Some(ref dest) = destination {
                                            // Only process messages from the specified destination
                                            if &from != dest {
                                                return Ok(());
                                            }
                                        }
                                        
                                        // Data should contain HDLC-encoded Reticulum packets
                                        let mut hdlc_rx_buffer = [0u8; BUFFER_SIZE];
                                        let mut rx_buffer = [0u8; BUFFER_SIZE + (BUFFER_SIZE / 2)];
                                        
                                        // Copy data to rx_buffer (starting from the end)
                                        let data_len = data.len();
                                        if data_len > 0 && data_len <= rx_buffer.len() {
                                            // Push data bytes into buffer
                                            for (i, &byte) in data.iter().enumerate() {
                                                if i < rx_buffer.len() {
                                                    rx_buffer[rx_buffer.len() - data_len + i] = byte;
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
                                                            log::trace!("i2p_interface: rx << ({}) {}", iface_address, packet);
                                                        }
                                                        let _ = rx_channel.send(RxMessage { address: iface_address, packet }).await;
                                                    } else {
                                                        log::warn!("i2p_interface: couldn't decode packet");
                                                    }
                                                } else {
                                                    log::warn!("i2p_interface: couldn't decode hdlc frame");
                                                }
                                            }
                                        }
                                        Ok(())
                                    }
                                    Ok(Err(e)) => {
                                        log::warn!("i2p_interface: receive error: {}", e);
                                        Err(e)
                                    }
                                    Err(e) => {
                                        log::warn!("i2p_interface: task join error: {}", e);
                                        Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                                    }
                                }
                            } => {
                                match result {
                                    Ok(_) => {},
                                    Err(_) => {
                                        // Connection error, break out of loop
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
                let destination = destination.clone();
                let session = session.clone();

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
                                    log::trace!("i2p_interface: tx >> ({}) {}", iface_address, packet);
                                }
                                let mut output = OutputBuffer::new(&mut tx_buffer);
                                if let Ok(_) = packet.serialize(&mut output) {
                                    let mut hdlc_output = OutputBuffer::new(&mut hdlc_tx_buffer[..]);

                                    if let Ok(_) = Hdlc::encode(output.as_slice(), &mut hdlc_output) {
                                        // Send over I2P network
                                        if let Some(ref dest) = destination {
                                            // Send to specific destination
                                            let session_clone = session.clone();
                                            let dest_clone = dest.clone();
                                            let data = hdlc_output.as_slice().to_vec();
                                            tokio::task::spawn_blocking(move || {
                                                let mut session_guard = session_clone.lock().unwrap();
                                                session_guard.send_msg(dest_clone, data);
                                            });
                                        } else {
                                            // Broadcast to all (not typical for I2P, but we can send to a broadcast address)
                                            // For now, we'll just log a warning
                                            log::warn!("i2p_interface: No destination specified, cannot send packet");
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

            log::info!("i2p_interface: disconnected from SAM bridge at {}:{}", sam_address, sam_port);
        }

        iface_stop.cancel();
    }

    #[cfg(not(feature = "i2p"))]
    pub async fn spawn(_context: InterfaceContext<I2PInterface>) {
        log::error!("i2p_interface: i2p feature not enabled");
    }
}

impl Interface for I2PInterface {
    fn mtu() -> usize {
        // I2P typical MTU is around 32KB, but we'll use a conservative value
        // I2P tunnel limit is around 61.5KB, but messages >31.5KB are not recommended
        16384
    }
}