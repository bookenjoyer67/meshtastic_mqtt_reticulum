use reticulum::iface::kiss::KissInterface;
use reticulum::transport::{Transport, TransportConfig};
use reticulum::destination::DestinationName;
use reticulum::identity::PrivateIdentity;
use rand_core::OsRng;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Check if kiss feature is enabled
    #[cfg(not(feature = "kiss"))]
    {
        eprintln!("KISS feature not enabled. Build with --features kiss");
        return Ok(());
    }

    // ---------- Reticulum setup ----------
    let mut transport = Transport::new(TransportConfig::default());
    
    // Get serial port from environment variable with fallback
    let serial_port = std::env::var("KISS_PORT")
        .unwrap_or_else(|_| "/dev/ttyUSB0".to_string());
    
    // Get baud rate from environment variable with fallback
    let baud_rate: u32 = std::env::var("KISS_BAUD_RATE")
        .unwrap_or_else(|_| "9600".to_string())
        .parse()
        .unwrap_or(9600);
    
    // Create KISS interface
    let kiss_iface = KissInterface::new(&serial_port, baud_rate);
    
    transport
        .iface_manager()
        .lock()
        .await
        .spawn(
            kiss_iface,
            KissInterface::spawn,
        );
    
    log::info!("Transport created with KISS interface on {} at {} baud", serial_port, baud_rate);

    let identity = PrivateIdentity::new_from_rand(OsRng);
    let dest_name = DestinationName::new("kiss_example", "app");
    let destination = transport.add_destination(identity, dest_name).await;
    let my_hash = destination.lock().await.desc.address_hash.clone();
    log::info!("My hash: {}", my_hash);

    let announce = destination.lock().await.announce(OsRng, None)?;
    transport.send_packet(announce).await;
    log::info!("Announced on KISS interface");

    // Keep running to receive packets
    log::info!("Listening for packets on KISS interface...");
    log::info!("Press Ctrl+C to exit");
    
    // Simple loop to keep the program running
    loop {
        sleep(Duration::from_secs(1)).await;
    }
}