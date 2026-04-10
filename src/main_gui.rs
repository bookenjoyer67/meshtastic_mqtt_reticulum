use meshtastic_reticulum_bridge::socket_bridge::spawn_socket_bridge;
use meshtastic_reticulum_bridge::mqtt::mqtt_task;
use meshtastic_reticulum_bridge::config::Config;
use meshtastic_reticulum_bridge::gui::MeshtasticGuiApp;

#[tokio::main]
async fn main() {
    env_logger::init();

    let (mqtt_gui_tx, mqtt_gui_rx) = tokio::sync::mpsc::unbounded_channel();
    let (gui_mqtt_tx, gui_mqtt_rx) = tokio::sync::mpsc::unbounded_channel();
    let mqtt_tx_clone = gui_mqtt_tx.clone();

    let config = Config::from_env();
    
    tokio::spawn(async move {
        if let Err(e) = mqtt_task(gui_mqtt_rx, mqtt_gui_tx, config).await {
            eprintln!("MQTT task error: {}", e);
        }
    });

    let (bridge_cmd_tx, bridge_cmd_rx) = tokio::sync::mpsc::unbounded_channel();
    let (bridge_event_tx, bridge_event_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        if let Err(e) = spawn_socket_bridge(bridge_cmd_rx, bridge_event_tx).await {
            eprintln!("Socket bridge error: {}", e);
        }
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Meshtastic + Reticulum Bridge",
        options,
        Box::new(|_cc| {
            Box::new(MeshtasticGuiApp::new(
                mqtt_tx_clone,
                mqtt_gui_rx,
                bridge_cmd_tx,
                bridge_event_rx,
                String::new(),
            ))
        }),
    )
    .expect("Failed to start GUI");
}