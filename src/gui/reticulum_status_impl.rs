//! Reticulum status and management UI implementation

use eframe::egui;
use crate::gui::app::MeshtasticGuiApp;
use crate::gui::BridgeCommand;

impl MeshtasticGuiApp {
    /// Show reticulum status panel
    pub fn reticulum_status_panel_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Reticulum Status");
        ui.separator();
        
        // Connection status
        ui.horizontal(|ui| {
            match &self.reticulum_status {
                crate::gui::ReticulumStatus::Disconnected => {
                    ui.colored_label(egui::Color32::RED, "❌ Disconnected");
                }
                crate::gui::ReticulumStatus::Connecting => {
                    ui.colored_label(egui::Color32::YELLOW, "🔄 Connecting...");
                }
                crate::gui::ReticulumStatus::Connected => {
                    ui.colored_label(egui::Color32::GREEN, "✅ Connected");
                }
                crate::gui::ReticulumStatus::Error(err) => {
                    ui.colored_label(egui::Color32::RED, format!("❌ Error: {}", err));
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.reticulum_connected {
                    if ui.button("🔌 Disconnect").clicked() {
                        let _ = self.disconnect_reticulum();
                    }
                } else {
                    if ui.button("🔗 Connect").clicked() {
                        let _ = self.connect_reticulum();
                    }
                }
            });
        });
        
        // My hash if connected
        if let Some(my_hash) = &self.reticulum_my_hash {
            ui.horizontal(|ui| {
                ui.label("My Hash:");
                ui.monospace(&my_hash[..16]);
                ui.label("...");
                if ui.small_button("📋").clicked() {
                    ui.output_mut(|o| o.copied_text = my_hash.clone());
                }
            });
        }
        
        ui.separator();
        
        // Statistics
        ui.heading("Statistics");
        ui.columns(2, |columns| {
            columns[0].label("Packets Sent:");
            columns[1].label(format!("{}", self.reticulum_stats.packets_sent));
            
            columns[0].label("Packets Received:");
            columns[1].label(format!("{}", self.reticulum_stats.packets_received));
            
            columns[0].label("Bytes Sent:");
            columns[1].label(format!("{} B", self.reticulum_stats.bytes_sent));
            
            columns[0].label("Bytes Received:");
            columns[1].label(format!("{} B", self.reticulum_stats.bytes_received));
            
            columns[0].label("Peers Discovered:");
            columns[1].label(format!("{}", self.reticulum_stats.peers_discovered));
            
            columns[0].label("Links Established:");
            columns[1].label(format!("{}", self.reticulum_stats.links_established));
            
            columns[0].label("Announces Sent:");
            columns[1].label(format!("{}", self.reticulum_stats.announces_sent));
            
            columns[0].label("Announces Received:");
            columns[1].label(format!("{}", self.reticulum_stats.announces_received));
        });
        
        ui.separator();
        
        // Interface status
        ui.heading("Interfaces");
        if self.interface_statuses.is_empty() {
            ui.label("No interfaces configured");
        } else {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (name, status) in &self.interface_statuses {
                        ui.horizontal(|ui| {
                            // Status indicator
                            if status.connected {
                                ui.colored_label(egui::Color32::GREEN, "●");
                            } else if status.enabled {
                                ui.colored_label(egui::Color32::YELLOW, "○");
                            } else {
                                ui.colored_label(egui::Color32::GRAY, "○");
                            }
                            
                            // Interface name and type
                            ui.label(format!("{} ({})", name, status.interface_type));
                            
                            // Stats
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(format!("↑{}B ↓{}B", status.bytes_sent, status.bytes_received));
                            });
                        });
                        
                        // Error if any
                        if let Some(error) = &status.error {
                            ui.colored_label(egui::Color32::RED, format!("  Error: {}", error));
                        }
                    }
                });
        }
        
        ui.separator();
        
        // Quick actions
        ui.heading("Quick Actions");
        ui.horizontal(|ui| {
            if ui.button("🔄 Refresh").clicked() {
                let _ = self.refresh_reticulum();
            }
            
            if ui.button("📊 Reset Stats").clicked() {
                self.reticulum_stats = crate::gui::ReticulumStats {
                    last_update: std::time::Instant::now(),
                    ..Default::default()
                };
            }
        });
    }
    
    /// Show reticulum peers panel
    pub fn reticulum_peers_panel_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Reticulum Peers");
        ui.separator();
        
        if self.reticulum_peers.is_empty() {
            ui.label("No peers discovered yet");
            ui.label("Make sure reticulum is connected and refresh to discover peers");
        } else {
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    // Collect hashes first to avoid borrowing issues
                    let hashes: Vec<String> = self.reticulum_peers.keys().cloned().collect();
                    
                    for hash in hashes {
                        ui.horizontal(|ui| {
                            ui.label("👤");
                            
                            // Hash display
                            ui.monospace(&hash[..16]);
                            ui.label("...");
                            
                            // Nickname editor
                            if let Some(nickname) = self.reticulum_peers.get_mut(&hash) {
                                ui.text_edit_singleline(nickname);
                            }
                            
                            // Actions
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("💬").clicked() {
                                    // Open message dialog
                                    self.selected_peer = Some(crate::gui::peers::Peer {
                                        main_hash: hash.clone(),
                                        file_hash: hash.clone(),
                                        name: None,
                                        last_seen: None,
                                        signal_strength: None,
                                        link_quality: None,
                                        interface: None,
                                        network_metrics: None,
                                        radio_metrics: None,
                                        last_metrics_update: None,
                                    });
                                }
                                
                                if ui.button("📁").clicked() {
                                    // Open file dialog
                                    self.selected_file_path = Some("".to_string());
                                    self.selected_peer = Some(crate::gui::peers::Peer {
                                        main_hash: hash.clone(),
                                        file_hash: hash.clone(),
                                        name: None,
                                        last_seen: None,
                                        signal_strength: None,
                                        link_quality: None,
                                        interface: None,
                                        network_metrics: None,
                                        radio_metrics: None,
                                        last_metrics_update: None,
                                    });
                                }
                            });
                        });
                    }
                });
        }
        
        ui.separator();
        
        // Send message to selected peer
        if let Some(peer) = &self.selected_peer {
            ui.heading(format!("Send to: {}", peer.main_hash));
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.input_text);
                if ui.button("Send").clicked() && !self.input_text.is_empty() {
                    let hash = peer.main_hash.clone();
                    let text = self.input_text.clone();
                    let bridge_cmd_tx = self.bridge_cmd_tx.clone();
                    tokio::spawn(async move {
                        // We need to check if reticulum is connected and send the message
                        // Since we can't access self here, we'll just send the command
                        // The actual connection check should be done elsewhere
                        let cmd = BridgeCommand::SendMessage {
                            dest_hash: hash,
                            text: text,
                        };
                        if let Err(e) = bridge_cmd_tx.send(cmd) {
                            eprintln!("Failed to send message: {}", e);
                        }
                    });
                    self.input_text.clear();
                }
            });
        }
    }
    
    /// Show reticulum configuration button
    pub fn show_reticulum_config_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("📡 Reticulum Config").clicked() {
            self.show_reticulum_config_window = true;
        }
    }
    
    /// Show reticulum status button
    pub fn show_reticulum_status_button(&mut self, ui: &mut egui::Ui) {
        let status_text = match &self.reticulum_status {
            crate::gui::ReticulumStatus::Disconnected => "❌ Reticulum",
            crate::gui::ReticulumStatus::Connecting => "🔄 Reticulum",
            crate::gui::ReticulumStatus::Connected => "✅ Reticulum",
            crate::gui::ReticulumStatus::Error(_) => "⚠️ Reticulum",
        };
        
        if ui.button(status_text).clicked() {
            // Toggle reticulum status panel visibility
            // This would need to be integrated with the main UI layout
        }
    }
}