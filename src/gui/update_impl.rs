use eframe::egui;
use crate::gui::app::MeshtasticGuiApp;

impl eframe::App for MeshtasticGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Initialize theme on first update
        if !self.messages.is_empty() && self.messages.len() == 1 {
            self.init_theme(ctx);
        }
        
        self.process_mqtt_events(ctx);
        self.process_reticulum_events(ctx);

        self.peer_list_panel(ctx);
        self.peer_details_panel(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Meshtastic + Reticulum Bridge");
            ui.separator();

            // Add configuration and theme buttons at the top
            ui.horizontal(|ui| {
                self.show_config_button(ui);
                ui.separator();
                self.show_reticulum_config_button(ui);
                ui.separator();
                self.show_search_button(ui);
                ui.separator();
                self.show_theme_button(ctx, ui);
            });
            ui.separator();

            // Reticulum status panel
            ui.collapsing("Reticulum Status", |ui| {
                self.reticulum_status_panel_ui(ui);
            });
            ui.separator();

            ui.collapsing("Channel Management", |ui| {
                self.channel_management_ui(ctx, ui);
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Send to channel:");
                egui::ComboBox::from_label("send_channel")
                    .selected_text(&self.active_channel)
                    .show_ui(ui, |ui| {
                        for ch in &self.channels {
                            ui.selectable_value(&mut self.active_channel, ch.clone(), ch);
                        }
                    });
            });
            ui.separator();

            ui.collapsing("Relay Settings", |ui| {
                self.relay_settings_ui(ctx, ui);
            });
            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Your nickname:");
                if ui.text_edit_singleline(&mut self.nickname).changed() {
                    self.save_nickname();
                }
            });
            ui.separator();

            if let Some(peer) = &self.selected_peer {
                ui.label(format!("Selected Reticulum peer: {}…", &peer.main_hash[..16]));
            } else {
                ui.label("No Reticulum peer selected.");
            }
            ui.separator();

            self.file_transfer_ui(ctx, ui);
            ui.separator();

            // Use filtered chat area instead of regular chat area
            self.chat_area_filtered(ui);
            ui.separator();

            self.message_input_ui(ctx, ui);
            
            // Add node list as a collapsible section
            ui.separator();
            ui.collapsing("Meshtastic Nodes", |ui| {
                if self.nodes.is_empty() {
                    ui.label("No nodes discovered yet.");
                    ui.label("(NodeInfo/Position packets will appear here)");
                } else {
                    for (id, node) in &self.nodes {
                        let short_id = if id.len() > 8 { &id[..8] } else { id };
                        ui.collapsing(format!("{} ({})", node.name, short_id), |ui| {
                            ui.label(format!("ID: {}", id));
                            if let (Some(lat), Some(lon)) = (node.latitude, node.longitude) {
                                ui.label(format!("Position: ({:.4}, {:.4})", lat, lon));
                            } else {
                                ui.label("Position: unknown");
                            }
                            if let Some(alt) = node.altitude {
                                ui.label(format!("Altitude: {:.1}m", alt));
                            }
                            ui.label(format!("Last seen: {}", node.last_seen.format("%H:%M:%S")));
                        });
                    }
                }
            });
        });

        // Show configuration window if needed
        if self.show_config_window {
            self.config_window_ui(ctx);
        }

        // Show search window if needed
        if self.show_search_panel {
            self.search_panel_ui(ctx);
        }

        // Show theme settings window if needed
        if self.show_theme_settings {
            self.theme_settings_ui(ctx);
        }

        // Show reticulum configuration window if needed
        if self.show_reticulum_config_window {
            self.reticulum_config_window_ui(ctx);
        }

        if self.show_qr_window {
            let mut close = false;
            egui::Window::new("Channel QR Code")
                .resizable(false)
                .show(ctx, |ui| {
                    if let Some(ref channel_name) = self.current_qr_channel {
                        ui.label(format!("Channel: {}", channel_name));
                    }
                    if let Some(ref texture) = self.qr_texture {
                        ui.add(egui::Image::new(texture).max_width(200.0).max_height(200.0));
                        ui.label("Scan with Meshtastic mobile app");
                    } else {
                        ui.label("QR code generation failed.");
                    }
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            if close {
                self.show_qr_window = false;
                self.current_qr_channel = None;
                self.qr_texture = None;
            }
        }
    }
}