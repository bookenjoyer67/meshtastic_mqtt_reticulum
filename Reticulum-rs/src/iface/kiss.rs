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

// KISS protocol constants
const FEND: u8 = 0xC0;    // Frame End
const FESC: u8 = 0xDB;    // Frame Escape
const TFEND: u8 = 0xDC;   // Transposed Frame End
const TFESC: u8 = 0xDD;   // Transposed Frame Escape

// KISS command bytes (only DATA frame type 0 is used for Reticulum)
const KISS_DATA_FRAME: u8 = 0x00;

pub struct KissInterface {
    port_name: String,
    baud_rate: u32,
}

impl KissInterface {
    pub fn new<T: Into<String>>(port_name: T, baud_rate: u32) -> Self {
        Self {
            port_name: port_name.into(),
            baud_rate,
        }
    }

    /// Encode data with KISS framing
    fn kiss_encode(data: &[u8], output: &mut Vec<u8>) {
        output.push(FEND);
        output.push(KISS_DATA_FRAME); // Port 0, DATA frame
        
        for &byte in data {
            match byte {
                FEND => {
                    output.push(FESC);
                    output.push(TFEND);
                }
                FESC => {
                    output.push(FESC);
                    output.push(TFESC);
                }
                _ => output.push(byte),
            }
        }
        
        output.push(FEND);
    }

    /// Decode KISS frame from buffer
    fn kiss_decode(buffer: &[u8]) -> Option<Vec<u8>> {
        if buffer.len() < 3 {
            return None; // Too short for a valid KISS frame
        }
        
        if buffer[0] != FEND || buffer[buffer.len() - 1] != FEND {
            return None; // Not properly framed
        }
        
        let command = buffer[1];
        if (command & 0x0F) != KISS_DATA_FRAME {
            return None; // Not a DATA frame
        }
        
        let mut result = Vec::new();
        let mut i = 2; // Skip FEND and command byte
        
        while i < buffer.len() - 1 { // Skip final FEND
            match buffer[i] {
                FESC => {
                    i += 1;
                    if i >= buffer.len() - 1 {
                        return None; // Incomplete escape sequence
                    }
                    match buffer[i] {
                        TFEND => result.push(FEND),
                        TFESC => result.push(FESC),
                        _ => return None, // Invalid escape sequence
                    }
                }
                byte => result.push(byte),
            }
            i += 1;
        }
        
        Some(result)
    }

    #[cfg(feature = "kiss")]
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
                log::info!("kiss_interface: couldn't open port <{}>", port_name);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }

            let cancel = context.cancel.clone();
            let stop = CancellationToken::new();

            let port = port_result.unwrap();
            
            // Configure port (serialport crate opens ports in exclusive mode by default)
            // No need to set_exclusive as it's already exclusive

            log::info!("kiss_interface opened port <{}> at {} baud", port_name, baud_rate);

            const BUFFER_SIZE: usize = core::mem::size_of::<Packet>() * 2;

            // Start receive task using spawn_blocking for synchronous I/O
            let rx_task = {
                let cancel = cancel.clone();
                let stop = stop.clone();
                let port_result = port.try_clone();
                
                if let Err(e) = port_result {
                    log::error!("kiss_interface: failed to clone port: {}", e);
                    continue;
                }
                
                let port = port_result.unwrap();
                let rx_channel = rx_channel.clone();

                tokio::spawn(async move {
                    let mut hdlc_rx_buffer = [0u8; BUFFER_SIZE];
                    let mut kiss_buffer = Vec::new();
                    let mut in_frame = false;
                    let mut escape_next = false;
                    let serial_buffer = [0u8; (BUFFER_SIZE * 16)];

                    loop {
                        // Check for cancellation
                        if cancel.is_cancelled() || stop.is_cancelled() {
                            break;
                        }

                        // Read from serial port using spawn_blocking
                        let read_result = tokio::task::spawn_blocking({
                            let mut port = port.try_clone().unwrap();
                            let mut serial_buffer = serial_buffer;
                            move || port.read(&mut serial_buffer)
                        }).await;

                        match read_result {
                            Ok(Ok(0)) => {
                                log::warn!("kiss_interface: port closed");
                                stop.cancel();
                                break;
                            }
                            Ok(Ok(n)) => {
                                // Process KISS frames from serial data
                                for i in 0..n {
                                    let byte = serial_buffer[i];
                                    
                                    if escape_next {
                                        escape_next = false;
                                        match byte {
                                            TFEND => kiss_buffer.push(FEND),
                                            TFESC => kiss_buffer.push(FESC),
                                            _ => {
                                                // Invalid escape sequence, discard frame
                                                in_frame = false;
                                                kiss_buffer.clear();
                                            }
                                        }
                                    } else if byte == FESC {
                                        escape_next = true;
                                    } else if byte == FEND {
                                        if in_frame {
                                            // End of frame
                                            in_frame = false;
                                            
                                            // Try to decode KISS frame
                                            if let Some(kiss_data) = Self::kiss_decode(&kiss_buffer) {
                                                // KISS data should contain HDLC-encoded Reticulum packets
                                                let mut rx_buffer = [0u8; BUFFER_SIZE + (BUFFER_SIZE / 2)];
                                                
                                                // Copy kiss_data to rx_buffer (starting from the end)
                                                let data_len = kiss_data.len();
                                                if data_len > 0 && data_len <= rx_buffer.len() {
                                                    // Push data bytes into buffer
                                                    for (i, &byte) in kiss_data.iter().enumerate() {
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
                                                                    log::trace!("kiss_interface: rx << ({}) {}", iface_address, packet);
                                                                }
                                                                let _ = rx_channel.send(RxMessage { address: iface_address, packet }).await;
                                                            } else {
                                                                log::warn!("kiss_interface: couldn't decode packet");
                                                            }
                                                        } else {
                                                            log::warn!("kiss_interface: couldn't decode hdlc frame");
                                                        }
                                                    }
                                                }
                                            }
                                            
                                            kiss_buffer.clear();
                                        } else {
                                            // Start of new frame
                                            in_frame = true;
                                            kiss_buffer.clear();
                                            kiss_buffer.push(FEND); // Add the starting FEND
                                        }
                                    } else if in_frame {
                                        kiss_buffer.push(byte);
                                    }
                                }
                            }
                            Ok(Err(e)) => {
                                log::warn!("kiss_interface: port error {}", e);
                                break;
                            }
                            Err(e) => {
                                log::warn!("kiss_interface: task error {}", e);
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
                                    log::trace!("kiss_interface: tx >> ({}) {}", iface_address, packet);
                                }
                                let mut output = OutputBuffer::new(&mut tx_buffer);
                                if let Ok(_) = packet.serialize(&mut output) {
                                    let mut hdlc_output = OutputBuffer::new(&mut hdlc_tx_buffer[..]);
                                    if let Ok(_) = Hdlc::encode(output.as_slice(), &mut hdlc_output) {
                                        // Encode with KISS framing
                                        let mut kiss_frame = Vec::new();
                                        Self::kiss_encode(hdlc_output.as_slice(), &mut kiss_frame);
                                        
                                        // Write to serial port using spawn_blocking
                                        let write_result = tokio::task::spawn_blocking({
                                            let mut port = port.try_clone().unwrap();
                                            let data = kiss_frame;
                                            move || {
                                                port.write_all(&data)?;
                                                port.flush()
                                            }
                                        }).await;

                                        if let Err(e) = write_result {
                                            log::warn!("kiss_interface: write error {}", e);
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

            log::info!("kiss_interface: closed port <{}>", port_name);
        }
    }

    #[cfg(not(feature = "kiss"))]
    pub async fn spawn(_context: InterfaceContext<Self>) {
        log::error!("kiss_interface: kiss feature not enabled");
    }
}

impl Interface for KissInterface {
    fn mtu() -> usize {
        2048
    }
}