use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::buffer::{InputBuffer, OutputBuffer};
use crate::error::RnsError;
use crate::iface::RxMessage;
use crate::packet::Packet;
use crate::serde::Serialize;

use super::hdlc::Hdlc;
use super::{Interface, InterfaceContext};

// TODO: Configure via features
const PACKET_TRACE: bool = false;

pub struct SerialInterface {
    port_name: String,
    baud_rate: u32,
}

impl SerialInterface {
    pub fn new<T: Into<String>>(port_name: T, baud_rate: u32) -> Self {
        Self {
            port_name: port_name.into(),
            baud_rate,
        }
    }

    #[cfg(feature = "serial")]
    pub async fn spawn(context: InterfaceContext<Self>) {
        let port_name = { context.inner.lock().unwrap().port_name.clone() };
        let baud_rate = { context.inner.lock().unwrap().baud_rate };
        let iface_address = context.channel.address;

        let (rx_channel, tx_channel) = context.channel.split();
        let tx_channel = Arc::new(tokio::sync::Mutex::new(tx_channel));

        loop {
            if context.cancel.is_cancelled() {
                break;
            }

            // Open serial port using serialport crate
            let port_result = serialport::new(&port_name, baud_rate)
                .open()
                .map_err(|e| RnsError::ConnectionError(format!("Failed to open serial port {}: {}", port_name, e)));

            if let Err(_) = port_result {
                log::info!("serial_interface: couldn't open port <{}>", port_name);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }

            let cancel = context.cancel.clone();
            let stop = CancellationToken::new();

            let port = port_result.unwrap();
            
            // Configure port (serialport crate opens ports in exclusive mode by default)
            // No need to set_exclusive as it's already exclusive

            log::info!("serial_interface opened port <{}> at {} baud", port_name, baud_rate);

            const BUFFER_SIZE: usize = core::mem::size_of::<Packet>() * 2;

            // Start receive task using spawn_blocking for synchronous I/O
            let rx_task = {
                let cancel = cancel.clone();
                let stop = stop.clone();
                let port_result = port.try_clone();
                
                if let Err(e) = port_result {
                    log::error!("serial_interface: failed to clone port: {}", e);
                    continue;
                }
                
                let port = port_result.unwrap();
                let rx_channel = rx_channel.clone();

                tokio::spawn(async move {
                    let mut hdlc_rx_buffer = [0u8; BUFFER_SIZE];
                    let mut rx_buffer = [0u8; BUFFER_SIZE + (BUFFER_SIZE / 2)];
                    let serial_buffer = [0u8; (BUFFER_SIZE * 16)];

                    loop {
                        // Check for cancellation
                        if cancel.is_cancelled() || stop.is_cancelled() {
                            break;
                        }

                        // Read from serial port using spawn_blocking
                        let read_result = tokio::task::spawn_blocking({
                            let mut port = port.try_clone().unwrap();
                            let mut serial_buffer = serial_buffer.clone();
                            move || port.read(&mut serial_buffer)
                        }).await;

                        match read_result {
                            Ok(Ok(0)) => {
                                log::warn!("serial_interface: port closed");
                                stop.cancel();
                                break;
                            }
                            Ok(Ok(n)) => {
                                // Serial stream may contain several or partial HDLC frames
                                for i in 0..n {
                                    // Push new byte from the end of buffer
                                    rx_buffer[BUFFER_SIZE-1] = serial_buffer[i];

                                    // Check if it contains a HDLC frame
                                    let frame = Hdlc::find(&rx_buffer[..]);
                                    if let Some(frame) = frame {
                                        // Decode HDLC frame and deserialize packet
                                        let frame_buffer = &mut rx_buffer[frame.0..frame.1+1];
                                        let mut output = OutputBuffer::new(&mut hdlc_rx_buffer[..]);
                                        if let Ok(_) = Hdlc::decode(frame_buffer, &mut output) {
                                            if let Ok(packet) = Packet::deserialize(&mut InputBuffer::new(output.as_slice())) {
                                                if PACKET_TRACE {
                                                    log::trace!("serial_interface: rx << ({}) {}", iface_address, packet);
                                                }
                                                let _ = rx_channel.send(RxMessage { address: iface_address, packet }).await;
                                            } else {
                                                log::warn!("serial_interface: couldn't decode packet");
                                            }
                                        } else {
                                            log::warn!("serial_interface: couldn't decode hdlc frame");
                                        }

                                        // Remove current HDLC frame data
                                        frame_buffer.fill(0);
                                    } else {
                                        // Move data left
                                        rx_buffer.copy_within(1.., 0);
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                log::warn!("serial_interface: port error {}", e);
                                break;
                            }
                            Err(e) => {
                                log::warn!("serial_interface: task error {}", e);
                                break;
                            }
                        }
                    }
                })
            };

            // Start transmit task using spawn_blocking for synchronous I/O
            let tx_task = {
                let cancel = cancel.clone();
                let tx_channel = tx_channel.clone();
                let port = port;

                tokio::spawn(async move {
                    loop {
                        if stop.is_cancelled() {
                            break;
                        }

                        let mut hdlc_tx_buffer = [0u8; BUFFER_SIZE];
                        let mut tx_buffer = [0u8; BUFFER_SIZE];

                        let mut tx_channel = tx_channel.lock().await;

                        // Use a timeout to check for cancellation
                        let message = tokio::time::timeout(
                            std::time::Duration::from_millis(100),
                            tx_channel.recv()
                        ).await;

                        if cancel.is_cancelled() || stop.is_cancelled() {
                            break;
                        }

                        match message {
                            Ok(Some(message)) => {
                                let packet = message.packet;
                                if PACKET_TRACE {
                                    log::trace!("serial_interface: tx >> ({}) {}", iface_address, packet);
                                }
                                let mut output = OutputBuffer::new(&mut tx_buffer);
                                if let Ok(_) = packet.serialize(&mut output) {
                                    let mut hdlc_output = OutputBuffer::new(&mut hdlc_tx_buffer[..]);
                                    if let Ok(_) = Hdlc::encode(output.as_slice(), &mut hdlc_output) {
                                        // Write to serial port using spawn_blocking
                                        let write_result = tokio::task::spawn_blocking({
                                            let mut port = port.try_clone().unwrap();
                                            let data = hdlc_output.as_slice().to_vec();
                                            move || {
                                                port.write_all(&data)?;
                                                port.flush()
                                            }
                                        }).await;

                                        if let Err(e) = write_result {
                                            log::warn!("serial_interface: write error {}", e);
                                        }
                                    }
                                }
                            }
                            Ok(None) => {
                                // Channel closed
                                break;
                            }
                            Err(_) => {
                                // Timeout, continue loop
                                continue;
                            }
                        }
                    }
                })
            };

            tx_task.await.unwrap();
            rx_task.await.unwrap();

            log::info!("serial_interface: closed port <{}>", port_name);
        }
    }

    #[cfg(not(feature = "serial"))]
    pub async fn spawn(_context: InterfaceContext<Self>) {
        log::error!("serial_interface: serial feature not enabled");
    }
}

impl Interface for SerialInterface {
    fn mtu() -> usize {
        2048
    }
}