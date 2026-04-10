use eframe::egui;
use crate::gui::BridgeCommand;
use crate::gui::app::MeshtasticGuiApp;
use crate::gui::peers::Peer;
use crate::gui::{SignalVisualizer, SignalStyle, NetworkMetricsVisualizer, InterfaceVisualizer};

fn strip_slashes(s: &str) -> String {
    s.trim_matches('/').to_string()
}

impl MeshtasticGuiApp {
    pub fn load_peers(&mut self) {
        if let Ok(data) = std::fs::read_to_string("peers.txt") {
            for line in data.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let main_clean = strip_slashes(parts[0]);
                    let file_clean = strip_slashes(parts[1]);
                    
                    // Parse additional fields if present (new format)
                    let mut last_seen = None;
                    let mut signal_strength = None;
                    let mut link_quality = None;
                    let mut interface = None;
                    let mut network_metrics = None;
                    let mut radio_metrics = None;
                    let mut last_metrics_update = None;
                    
                    if parts.len() >= 3 && !parts[2].is_empty() {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(parts[2]) {
                            last_seen = Some(dt.with_timezone(&chrono::Local));
                        }
                    }
                    
                    if parts.len() >= 4 && !parts[3].is_empty() {
                        if let Ok(signal) = parts[3].parse::<i32>() {
                            signal_strength = Some(signal);
                        }
                    }
                    
                    if parts.len() >= 5 && !parts[4].is_empty() {
                        if let Ok(quality) = parts[4].parse::<u8>() {
                            link_quality = Some(quality);
                        }
                    }
                    
                    if parts.len() >= 6 && !parts[5].is_empty() {
                        interface = Some(parts[5].to_string());
                    }
                    
                    // Parse network metrics (JSON) if present
                    if parts.len() >= 7 && !parts[6].is_empty() {
                        if let Ok(metrics) = serde_json::from_str::<crate::network_metrics::NetworkMetrics>(parts[6]) {
                            network_metrics = Some(metrics);
                        }
                    }
                    
                    // Parse radio metrics (JSON) if present
                    if parts.len() >= 8 && !parts[7].is_empty() {
                        if let Ok(metrics) = serde_json::from_str::<crate::gui::peers::RadioMetrics>(parts[7]) {
                            radio_metrics = Some(metrics);
                        }
                    }
                    
                    // Parse last metrics update timestamp
                    if parts.len() >= 9 && !parts[8].is_empty() {
                        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(parts[8]) {
                            last_metrics_update = Some(dt.with_timezone(&chrono::Local));
                        }
                    }
                    
                    let peer = Peer { 
                        main_hash: main_clean.clone(), 
                        file_hash: file_clean,
                        name: None,
                        last_seen,
                        signal_strength,
                        link_quality,
                        interface,
                        network_metrics,
                        radio_metrics,
                        last_metrics_update,
                    };
                    if !self.peers.iter().any(|p| p.main_hash == peer.main_hash) {
                        self.peers.push(peer);
                    }
                }
            }
        }
    }

    pub fn save_peer(&self, main: &str, file: &str) {
        // Old format for backward compatibility
        let main_clean = strip_slashes(main);
        let file_clean = strip_slashes(file);
        let line = format!("{},{}\n", main_clean, file_clean);
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("peers.txt")
            .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
    }
    
    pub fn save_peer_with_metadata(&self, peer: &Peer) {
        let main_clean = strip_slashes(&peer.main_hash);
        let file_clean = strip_slashes(&peer.file_hash);
        
        let last_seen_str = peer.last_seen
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        
        let signal_str = peer.signal_strength
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let quality_str = peer.link_quality
            .map(|q| q.to_string())
            .unwrap_or_default();
        
        let interface_str = peer.interface
            .as_deref()
            .unwrap_or("");
        
        // Serialize network metrics to JSON
        let network_metrics_str = peer.network_metrics
            .as_ref()
            .and_then(|m| serde_json::to_string(m).ok())
            .unwrap_or_default();
        
        // Serialize radio metrics to JSON
        let radio_metrics_str = peer.radio_metrics
            .as_ref()
            .and_then(|m| serde_json::to_string(m).ok())
            .unwrap_or_default();
        
        let last_metrics_update_str = peer.last_metrics_update
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_default();
        
        let line = format!("{},{},{},{},{},{},{},{},{}\n", 
            main_clean, file_clean, last_seen_str, signal_str, quality_str, 
            interface_str, network_metrics_str, radio_metrics_str, last_metrics_update_str);
        
        // Read existing file, update if peer exists, otherwise append
        let mut updated = false;
        let mut lines = Vec::new();
        
        if let Ok(data) = std::fs::read_to_string("peers.txt") {
            for existing_line in data.lines() {
                if existing_line.starts_with(&format!("{},", main_clean)) {
                    lines.push(line.clone());
                    updated = true;
                } else {
                    lines.push(existing_line.to_string());
                }
            }
        }
        
        if !updated {
            lines.push(line);
        }
        
        let _ = std::fs::write("peers.txt", lines.join("\n"));
    }

    pub fn select_peer(&mut self, peer: Peer, ctx: &egui::Context) {
        self.selected_peer = Some(peer);
        ctx.request_repaint();
    }

    pub fn load_nicknames(&mut self) {
        if let Ok(data) = std::fs::read_to_string("nicknames.txt") {
            for line in data.lines() {
                if let Some((hash, nick)) = line.split_once(',') {
                    self.peer_nicknames.insert(hash.to_string(), nick.to_string());
                }
            }
        }
    }

    pub fn set_peer_nickname(&mut self, hash: &str, nickname: &str, ctx: &egui::Context) {
        if nickname.is_empty() {
            self.peer_nicknames.remove(hash);
        } else {
            self.peer_nicknames.insert(hash.to_string(), nickname.to_string());
        }
        let mut content = String::new();
        for (h, n) in &self.peer_nicknames {
            content.push_str(&format!("{},{}\n", h, n));
        }
        let _ = std::fs::write("nicknames.txt", content);
        ctx.request_repaint();
    }

    pub fn peer_details_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("peer_details")
            .default_width(400.0)
            .show(ctx, |ui| {
                ui.heading("Peer Details");
                ui.separator();
                
                if let Some(peer) = &self.selected_peer {
                    // Get nickname if available
                    let nickname = self.peer_nicknames.get(&peer.main_hash).cloned();
                    
                    // Clone the hash for use in closures
                    let peer_hash = peer.main_hash.clone();
                    
                    ui.label(format!("Hash: {}", peer.main_hash));
                    ui.label(format!("File hash: {}", peer.file_hash));
                    
                    if let Some(nick) = nickname {
                        ui.label(format!("Nickname: {}", nick));
                    } else {
                        ui.label("Nickname: —");
                    }
                    
                    if let Some(last_seen) = peer.last_seen {
                        ui.label(format!("Last seen: {}", last_seen.format("%Y-%m-%d %H:%M:%S")));
                    } else {
                        ui.label("Last seen: never");
                    }
                    
                    // Display signal strength with visual indicator
                    ui.horizontal(|ui| {
                        ui.label("Signal:");
                        let signal_viz = SignalVisualizer::new()
                            .with_style(SignalStyle::Bars)
                            .with_size(16.0)
                            .with_text(true);
                        signal_viz.draw(ui, peer.signal_strength, peer.link_quality);
                    });
                    
                    // Display interface with icon
                    if let Some(interface) = &peer.interface {
                        ui.horizontal(|ui| {
                            ui.label("Interface:");
                            let iface_viz = InterfaceVisualizer::new()
                                .compact()
                                .with_icon(true)
                                .with_name(true);
                            iface_viz.draw(ui, interface);
                        });
                    }
                    
                    // Display network metrics if available
                    ui.separator();
                    ui.heading("Network Metrics");
                    
                    // Check if we have actual metrics
                    let has_actual_metrics = peer.network_metrics.is_some() || peer.radio_metrics.is_some();
                    
                    if has_actual_metrics {
                        // Display actual network metrics if available
                        if let Some(metrics) = &peer.network_metrics {
                            ui.colored_label(egui::Color32::GREEN, "📊 Actual Network Metrics");
                            
                            let metrics_viz = NetworkMetricsVisualizer::new()
                                .with_latency(true)
                                .with_packet_loss(true)
                                .with_quality(true);
                            
                            metrics_viz.draw(ui, 
                                Some(metrics.latency_ms), 
                                Some(metrics.packet_loss_percent), 
                                Some(metrics.quality_score));
                            
                            ui.label(format!("Bandwidth: {:.1} kbps", metrics.bandwidth_kbps));
                            ui.label(format!("Jitter: {:.1} ms", metrics.jitter_ms));
                            if let Some(last_update) = peer.last_metrics_update {
                                ui.label(format!("Last updated: {}", last_update.format("%Y-%m-%d %H:%M:%S")));
                            }
                        }
                        
                        // Display radio metrics if available
                        if let Some(radio) = &peer.radio_metrics {
                            ui.separator();
                            ui.colored_label(egui::Color32::BLUE, "📡 Radio Metrics");
                            
                            ui.horizontal(|ui| {
                                ui.label("RSSI:");
                                ui.label(format!("{} dBm", radio.rssi_dbm));
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("SNR:");
                                ui.label(format!("{:.1} dB", radio.snr_db));
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Packets:");
                                ui.label(format!("{} received, {} lost", radio.packet_count, radio.packet_loss_count));
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Link Quality:");
                                let signal_viz = SignalVisualizer::new()
                                    .with_style(SignalStyle::Bars)
                                    .with_size(12.0)
                                    .with_text(true);
                                signal_viz.draw(ui, Some(radio.rssi_dbm), Some(radio.link_quality));
                            });
                            
                            ui.label(format!("Updated: {}", radio.timestamp.format("%Y-%m-%d %H:%M:%S")));
                        }
                    } else {
                        // Show simulated metrics with disclaimer
                        ui.colored_label(egui::Color32::YELLOW, "⚠️ Simulated Metrics");
                        ui.label("Actual network metrics collection not yet implemented.");
                        ui.label("These values are simulated for demonstration.");
                        
                        let (latency, packet_loss, quality) = self.simulate_network_metrics(peer);
                        
                        let metrics_viz = NetworkMetricsVisualizer::new()
                            .with_latency(true)
                            .with_packet_loss(true)
                            .with_quality(true);
                        
                        metrics_viz.draw(ui, latency, packet_loss, quality);
                        
                        ui.separator();
                        ui.label("To implement actual metrics:");
                        ui.label("1. Enable Reticulum connection");
                        ui.label("2. Send periodic test packets");
                        ui.label("3. Measure round-trip times");
                        ui.label("4. Track packet delivery success");
                    }
                    
                    ui.separator();
                    
                    // Add action buttons
                    ui.horizontal(|ui| {
                        if ui.button("📋 Copy Hash").clicked() {
                            ui.output_mut(|o| o.copied_text = peer_hash.clone());
                        }
                        
                        if ui.button("🗑️ Remove").clicked() {
                            self.peers.retain(|p| p.main_hash != peer_hash);
                            self.selected_peer = None;
                            ctx.request_repaint();
                        }
                    });
                } else {
                    ui.label("No peer selected.");
                    ui.label("Click on a peer in the left panel to see details.");
                }
            });
    }
    
    /// Simulate network metrics based on peer information
    fn simulate_network_metrics(&self, peer: &Peer) -> (Option<f32>, Option<f32>, Option<u8>) {
        // This is a simulation - in a real implementation, we would collect actual metrics
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let interface_type = peer.interface.as_deref().unwrap_or("Unknown");
        
        match interface_type {
            s if s.contains("TCP") || s.contains("UDP") => {
                // Network interfaces: simulate latency and packet loss
                let latency = Some(rng.gen_range(10.0..200.0));
                let packet_loss = Some(rng.gen_range(0.0..5.0));
                let quality = Some(rng.gen_range(70..100));
                (latency, packet_loss, quality)
            }
            s if s.contains("LoRa") || s.contains("Radio") => {
                // Radio interfaces: simulate based on signal strength
                let latency = Some(rng.gen_range(100.0..500.0));
                let packet_loss = match peer.signal_strength {
                    Some(rssi) if rssi >= -70 => rng.gen_range(0.0..1.0),
                    Some(rssi) if rssi >= -85 => rng.gen_range(1.0..5.0),
                    Some(rssi) if rssi >= -100 => rng.gen_range(5.0..15.0),
                    Some(_) => rng.gen_range(15.0..30.0),
                    None => rng.gen_range(10.0..25.0),
                };
                let packet_loss = Some(packet_loss);
                let quality = peer.link_quality;
                (latency, packet_loss, quality)
            }
            _ => {
                // Default simulation
                let latency = Some(rng.gen_range(50.0..300.0));
                let packet_loss = Some(rng.gen_range(0.0..10.0));
                let quality = Some(rng.gen_range(50..90));
                (latency, packet_loss, quality)
            }
        }
    }

    pub fn peer_list_panel(&mut self, ctx: &egui::Context) {
        let mut clicked_peer = None;
        let mut editing_peer: Option<String> = None;
        let mut temp_nickname = String::new();

        egui::SidePanel::left("peer_list")
            .default_width(350.0)
            .show(ctx, |ui| {
                ui.heading("Reticulum Peers");
                ui.separator();

                if ui.button("🔄 Refresh Announce").clicked() {
                    let _ = self.bridge_cmd_tx.send(BridgeCommand::Refresh);
                }
                ui.separator();

                if ui.button("🗑️ Clear Peers").clicked() {
                    self.peers.clear();
                    self.selected_peer = None;
                    ctx.request_repaint();
                }
                ui.separator();

                egui::ScrollArea::vertical()
                    .max_height(500.0)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        for peer in &self.peers {
                            let hash = &peer.main_hash;
                            let display_name = self.peer_nicknames
                                .get(hash)
                                .cloned()
                                .unwrap_or_else(|| format!("{}…", &hash[..16]));
                            
                            // Create tooltip with additional information
                            let mut tooltip_text = format!(
                                "Hash: {}\n\
                                 Last seen: {}\n\
                                 Signal: {} dBm\n\
                                 Link quality: {}\n\
                                 Interface: {}",
                                hash,
                                peer.last_seen.map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or("never".to_string()),
                                peer.signal_strength.map(|s| s.to_string()).unwrap_or("N/A".to_string()),
                                peer.link_quality.map(|q| format!("{}%", q)).unwrap_or("N/A".to_string()),
                                peer.interface.as_deref().unwrap_or("Reticulum Network")
                            );
                            
                            // Add metrics information to tooltip if available
                            if let Some(metrics) = &peer.network_metrics {
                                tooltip_text.push_str(&format!(
                                    "\n\n📊 Network Metrics:\n\
                                     Latency: {:.1} ms\n\
                                     Packet Loss: {:.1}%\n\
                                     Quality: {}%",
                                    metrics.latency_ms,
                                    metrics.packet_loss_percent,
                                    metrics.quality_score
                                ));
                            }
                            
                            if let Some(radio) = &peer.radio_metrics {
                                tooltip_text.push_str(&format!(
                                    "\n\n📡 Radio Metrics:\n\
                                     RSSI: {} dBm\n\
                                     SNR: {:.1} dB\n\
                                     Packets: {} received, {} lost",
                                    radio.rssi_dbm,
                                    radio.snr_db,
                                    radio.packet_count,
                                    radio.packet_loss_count
                                ));
                            }
                            
                            ui.horizontal(|ui| {
                                // Add signal strength indicator before peer name
                                let signal_viz = SignalVisualizer::new()
                                    .with_style(SignalStyle::Bars)
                                    .with_size(12.0)
                                    .with_text(false)
                                    .with_tooltip(false);
                                signal_viz.draw(ui, peer.signal_strength, peer.link_quality);
                                
                                let response = ui.add(egui::Button::new(&display_name).wrap(false));
                                if response.on_hover_text(tooltip_text).clicked() {
                                    clicked_peer = Some(peer.clone());
                                }
                                if ui.small_button("✏️").clicked() {
                                    editing_peer = Some(hash.clone());
                                    temp_nickname = self.peer_nicknames.get(hash).cloned().unwrap_or_default();
                                }
                            });
                        }
                        if self.peers.is_empty() {
                            ui.label("No peers discovered.");
                            ui.label("(Announces will appear here)");
                        }
                    });
            });

        if let Some(peer) = clicked_peer {
            self.select_peer(peer, ctx);
        }

        if let Some(hash) = editing_peer.take() {
            let mut nick = temp_nickname;
            egui::Window::new("Edit Nickname")
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label(format!("Peer: {}", &hash[..16]));
                    ui.text_edit_singleline(&mut nick);
                    if ui.button("Save").clicked() {
                        self.set_peer_nickname(&hash, &nick, ctx);
                    }
                    if ui.button("Cancel").clicked() {
                        // do nothing
                    }
                });
        }
    }
}