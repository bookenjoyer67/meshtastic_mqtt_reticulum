//! Reticulum configuration UI implementation

use eframe::egui;
use crate::gui::app::MeshtasticGuiApp;
use crate::gui::reticulum_config::*;

impl MeshtasticGuiApp {
    /// Show reticulum configuration window
    pub fn reticulum_config_window_ui(&mut self, ctx: &egui::Context) {
        let mut close_window = false;
        let mut save_requested = false;
        let mut load_requested = false;
        
        let mut window_open = self.show_reticulum_config_window;
        egui::Window::new("Reticulum Configuration")
            .resizable(true)
            .default_width(800.0)
            .default_height(600.0)
            .open(&mut window_open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(450.0)
                    .show(ui, |ui| {
                        self.reticulum_config_ui(ui);
                    });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("💾 Save").clicked() {
                        save_requested = true;
                    }
                    
                    if ui.button("📂 Load").clicked() {
                        load_requested = true;
                    }
                    
                    if ui.button("❌ Close").clicked() {
                        close_window = true;
                    }
                });
            });
        
        self.show_reticulum_config_window = window_open;
        
        if close_window {
            self.show_reticulum_config_window = false;
        }
        
        if save_requested {
            if let Err(e) = self.reticulum_config.save_to_file("reticulum_config.toml") {
                self.messages.push(("System".to_string(), format!("Failed to save reticulum config: {}", e)));
            } else {
                self.messages.push(("System".to_string(), "Reticulum configuration saved".to_string()));
            }
        }
        
        if load_requested {
            if let Ok(config) = ReticulumConfig::load_from_file("reticulum_config.toml") {
                self.reticulum_config = config;
                self.messages.push(("System".to_string(), "Reticulum configuration loaded".to_string()));
            } else {
                self.messages.push(("System".to_string(), "Failed to load reticulum config".to_string()));
            }
        }
    }
    
    /// Main reticulum configuration UI
    fn reticulum_config_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Global Configuration");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Node Name:");
            ui.text_edit_singleline(&mut self.reticulum_config.global.node_name);
        });
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.reticulum_config.global.enable_forwarding, "Enable Forwarding");
            ui.checkbox(&mut self.reticulum_config.global.enable_announces, "Enable Announces");
        });
        
        ui.horizontal(|ui| {
            ui.label("Max Packet Size:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.global.max_packet_size).clamp_range(100..=5000));
            ui.label("bytes");
        });
        
        ui.horizontal(|ui| {
            ui.label("Default MTU:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.global.default_mtu).clamp_range(100..=1500));
            ui.label("bytes");
        });
        
        ui.separator();
        ui.heading("Interfaces");
        ui.separator();
        
        self.interfaces_ui(ui);
        
        ui.separator();
        ui.heading("Transport Settings");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Max Hops:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.transport.max_hops).clamp_range(1..=255));
        });
        
        ui.horizontal(|ui| {
            ui.label("Path Request Timeout:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.transport.path_request_timeout).clamp_range(1..=300));
            ui.label("seconds");
        });
        
        ui.horizontal(|ui| {
            ui.label("Link Establish Timeout:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.transport.link_establish_timeout).clamp_range(1..=300));
            ui.label("seconds");
        });
        
        ui.horizontal(|ui| {
            ui.label("Packet Cache Size:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.transport.packet_cache_size).clamp_range(64..=4096));
        });
        
        ui.horizontal(|ui| {
            ui.label("Announce Table Size:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.transport.announce_table_size).clamp_range(64..=2048));
        });
        
        ui.horizontal(|ui| {
            ui.label("Link Table Size:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.transport.link_table_size).clamp_range(32..=1024));
        });
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.reticulum_config.transport.eager_rerouting, "Eager Rerouting");
            ui.checkbox(&mut self.reticulum_config.transport.restart_links, "Restart Links");
        });
        
        ui.separator();
        ui.heading("Security Settings");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.reticulum_config.security.enable_encryption, "Enable Encryption");
            ui.checkbox(&mut self.reticulum_config.security.verify_signatures, "Verify Signatures");
        });
        
        ui.horizontal(|ui| {
            ui.label("Min Key Strength:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.security.min_key_strength).clamp_range(128..=512));
            ui.label("bits");
        });
        
        ui.horizontal(|ui| {
            ui.label("Certificate Validation:");
            ui.text_edit_singleline(&mut self.reticulum_config.security.cert_validation);
        });
        
        ui.separator();
        ui.heading("Performance Settings");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Thread Pool Size:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.performance.thread_pool_size).clamp_range(1..=16));
        });
        
        ui.horizontal(|ui| {
            ui.label("Max Concurrent Connections:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.performance.max_concurrent_connections).clamp_range(10..=1000));
        });
        
        ui.horizontal(|ui| {
            ui.label("Connection Pool Size:");
            ui.add(egui::DragValue::new(&mut self.reticulum_config.performance.connection_pool_size).clamp_range(5..=100));
        });
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.reticulum_config.performance.enable_compression, "Enable Compression");
            if self.reticulum_config.performance.enable_compression {
                ui.label("Level:");
                ui.add(egui::DragValue::new(&mut self.reticulum_config.performance.compression_level).clamp_range(1..=9));
            }
        });
        
        ui.separator();
        ui.heading("Logging Settings");
        ui.separator();
        
        ui.horizontal(|ui| {
            ui.label("Log Level:");
            egui::ComboBox::from_id_source("log_level")
                .selected_text(&self.reticulum_config.logging.level)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.reticulum_config.logging.level, "Trace".to_string(), "Trace");
                    ui.selectable_value(&mut self.reticulum_config.logging.level, "Debug".to_string(), "Debug");
                    ui.selectable_value(&mut self.reticulum_config.logging.level, "Info".to_string(), "Info");
                    ui.selectable_value(&mut self.reticulum_config.logging.level, "Warn".to_string(), "Warn");
                    ui.selectable_value(&mut self.reticulum_config.logging.level, "Error".to_string(), "Error");
                });
        });
        
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.reticulum_config.logging.enable_file_logging, "Enable File Logging");
            ui.checkbox(&mut self.reticulum_config.logging.enable_console, "Enable Console");
            ui.checkbox(&mut self.reticulum_config.logging.structured_logging, "Structured Logging");
        });
        
        if self.reticulum_config.logging.enable_file_logging {
            ui.horizontal(|ui| {
                ui.label("Log File:");
                ui.text_edit_singleline(&mut self.reticulum_config.logging.log_file);
            });
            
            ui.horizontal(|ui| {
                ui.label("Max Log Size:");
                ui.add(egui::DragValue::new(&mut self.reticulum_config.logging.max_log_size_mb).clamp_range(1..=100));
                ui.label("MB");
            });
            
            ui.horizontal(|ui| {
                ui.label("Max Log Files:");
                ui.add(egui::DragValue::new(&mut self.reticulum_config.logging.max_log_files).clamp_range(1..=50));
            });
        }
        
        ui.horizontal(|ui| {
            ui.label("Log Format:");
            egui::ComboBox::from_id_source("log_format")
                .selected_text(&self.reticulum_config.logging.format)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.reticulum_config.logging.format, "Text".to_string(), "Text");
                    ui.selectable_value(&mut self.reticulum_config.logging.format, "JSON".to_string(), "JSON");
                });
        });
    }
    
    /// Interfaces configuration UI
    fn interfaces_ui(&mut self, ui: &mut egui::Ui) {
        // Interface list
        ui.horizontal(|ui| {
            ui.label("Interfaces:");
            if ui.button("➕ Add Interface").clicked() {
                self.show_add_interface_dialog();
            }
        });
        
        // List existing interfaces
        let mut interface_to_remove = None;
        let mut interface_to_edit = None;
        
        for (i, interface) in self.reticulum_config.interfaces.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.checkbox(&mut interface.enabled, "");
                
                let display_text = if interface.enabled {
                    format!("✅ {}", interface.display_name())
                } else {
                    format!("❌ {}", interface.display_name())
                };
                
                if ui.selectable_label(false, &display_text).clicked() {
                    interface_to_edit = Some(i);
                }
                
                if ui.button("🗑️").clicked() {
                    interface_to_remove = Some(i);
                }
            });
        }
        
        // Handle interface removal
        if let Some(index) = interface_to_remove {
            self.reticulum_config.interfaces.remove(index);
        }
        
        // Handle interface editing
        if let Some(index) = interface_to_edit {
            self.selected_interface = Some(self.reticulum_config.interfaces[index].name.clone());
        }
        
        // Show interface editor if an interface is selected
        if let Some(interface_name) = &self.selected_interface {
            // Get the interface index first
            let interface_index = self.reticulum_config.interfaces.iter()
                .position(|iface| &iface.name == interface_name);
            
            if let Some(index) = interface_index {
                ui.separator();
                ui.heading(format!("Edit Interface: {}", self.reticulum_config.interfaces[index].display_name()));
                
                // Create a mutable reference to the interface
                let interface = &mut self.reticulum_config.interfaces[index];
                
                // Call the editor UI - we need to pass self as immutable
                let close_editor = Self::interface_editor_ui_static(ui, interface);
                if close_editor {
                    self.selected_interface = None;
                }
            }
        }
    }
    
    /// Show add interface dialog
    fn show_add_interface_dialog(&mut self) {
        // For now, just add a default interface
        let new_interface = match self.new_interface_type {
            InterfaceType::TcpClient => InterfaceConfig::new_tcp_client(
                &format!("tcp-client-{}", self.reticulum_config.interfaces.len() + 1),
                "localhost",
                4242,
            ),
            InterfaceType::TcpServer => InterfaceConfig::new_tcp_server(
                &format!("tcp-server-{}", self.reticulum_config.interfaces.len() + 1),
                "0.0.0.0",
                4242,
            ),
            InterfaceType::Udp => InterfaceConfig::new_udp(
                &format!("udp-{}", self.reticulum_config.interfaces.len() + 1),
                "0.0.0.0",
                4243,
            ),
            InterfaceType::Serial => InterfaceConfig::new_serial(
                &format!("serial-{}", self.reticulum_config.interfaces.len() + 1),
                "/dev/ttyUSB0",
                115200,
            ),
            InterfaceType::Mqtt => InterfaceConfig::new_mqtt(
                &format!("mqtt-{}", self.reticulum_config.interfaces.len() + 1),
                "localhost",
                1883,
            ),
            InterfaceType::Kiss => InterfaceConfig::new_kiss(
                &format!("kiss-{}", self.reticulum_config.interfaces.len() + 1),
                "/dev/ttyUSB1",
                9600,
            ),
            InterfaceType::I2p => InterfaceConfig::new_i2p(
                &format!("i2p-{}", self.reticulum_config.interfaces.len() + 1),
            ),
        };
        
        self.reticulum_config.add_interface(new_interface);
    }
    
    /// Interface editor UI (static version that doesn't need self)
    fn interface_editor_ui_static(ui: &mut egui::Ui, interface: &mut InterfaceConfig) -> bool {
        let mut close_editor = false;
        
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut interface.name);
        });
        
        ui.horizontal(|ui| {
            ui.label("Type:");
            let type_str = interface.interface_type.as_str();
            ui.label(type_str);
        });
        
        ui.checkbox(&mut interface.enabled, "Enabled");
        
        match interface.interface_type {
            InterfaceType::TcpClient => {
                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut interface.host);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.add(egui::DragValue::new(&mut interface.port).clamp_range(1..=65535));
                });
            }
            InterfaceType::TcpServer | InterfaceType::Udp => {
                ui.horizontal(|ui| {
                    ui.label("Bind Address:");
                    ui.text_edit_singleline(&mut interface.bind_address);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.add(egui::DragValue::new(&mut interface.port).clamp_range(1..=65535));
                });
            }
            InterfaceType::Serial | InterfaceType::Kiss => {
                ui.horizontal(|ui| {
                    ui.label("Serial Port:");
                    ui.text_edit_singleline(&mut interface.serial_port);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Baud Rate:");
                    ui.add(egui::DragValue::new(&mut interface.baud_rate).clamp_range(300..=115200));
                });
            }
            InterfaceType::Mqtt => {
                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut interface.host);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.add(egui::DragValue::new(&mut interface.port).clamp_range(1..=65535));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Client ID:");
                    ui.text_edit_singleline(&mut interface.mqtt_client_id);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Topic Prefix:");
                    ui.text_edit_singleline(&mut interface.mqtt_topic_prefix);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut interface.mqtt_username);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.text_edit_singleline(&mut interface.mqtt_password);
                });
                
                ui.checkbox(&mut interface.mqtt_use_tls, "Use TLS");
            }
            InterfaceType::I2p => {
                ui.horizontal(|ui| {
                    ui.label("SAM Address:");
                    ui.text_edit_singleline(&mut interface.i2p_sam_address);
                });
                
                ui.horizontal(|ui| {
                    ui.label("SAM Port:");
                    ui.add(egui::DragValue::new(&mut interface.i2p_sam_port).clamp_range(1..=65535));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Destination:");
                    ui.text_edit_singleline(&mut interface.i2p_destination);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Session Name:");
                    ui.text_edit_singleline(&mut interface.i2p_session_name);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Session Type:");
                    ui.text_edit_singleline(&mut interface.i2p_session_type);
                });
                
                ui.checkbox(&mut interface.i2p_use_local_dest, "Use Local Destination");
                
                ui.horizontal(|ui| {
                    ui.label("Max Connection Attempts:");
                    ui.add(egui::DragValue::new(&mut interface.i2p_max_connection_attempts).clamp_range(1..=100));
                });
            }
        }
        
        ui.separator();
        if ui.button("Close Editor").clicked() {
            close_editor = true;
        }
        
        close_editor
    }

}