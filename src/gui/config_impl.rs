use eframe::egui;
use crate::gui::app::MeshtasticGuiApp;

impl MeshtasticGuiApp {
    pub fn config_window_ui(&mut self, ctx: &egui::Context) {
        let mut close = false;
        let mut save_requested = false;
        let mut load_requested = false;
        
        // Create local copies of fields to avoid borrowing issues
        let mut mqtt_username = self.mqtt_username.clone();
        let mut mqtt_password = self.mqtt_password.clone();
        let mut mqtt_host = self.mqtt_host.clone();
        let mut mqtt_port = self.mqtt_port.clone();
        let mut mqtt_use_tls = self.mqtt_use_tls;
        let mut reticulum_server = self.reticulum_server.clone();
        
        egui::Window::new("Configuration")
            .resizable(true)
            .default_width(500.0)
            .open(&mut self.show_config_window)
            .show(ctx, |ui| {
                ui.heading("MQTT Configuration");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut mqtt_username);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.text_edit_singleline(&mut mqtt_password);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut mqtt_host);
                });
                
                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut mqtt_port);
                });
                
                ui.horizontal(|ui| {
                    ui.checkbox(&mut mqtt_use_tls, "Use TLS");
                });
                
                ui.separator();
                ui.heading("Reticulum Configuration");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Server:");
                    ui.text_edit_singleline(&mut reticulum_server);
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        save_requested = true;
                        close = true;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                    
                    if ui.button("Load").clicked() {
                        load_requested = true;
                    }
                });
                
                ui.separator();
                ui.label("Note: Changes will take effect after restarting the application.");
            });
        
        // Handle save/load requests outside the closure
        if save_requested {
            // Update the actual fields
            self.mqtt_username = mqtt_username;
            self.mqtt_password = mqtt_password;
            self.mqtt_host = mqtt_host;
            self.mqtt_port = mqtt_port;
            self.mqtt_use_tls = mqtt_use_tls;
            self.reticulum_server = reticulum_server;
            
            self.save_config();
            self.messages.push(("System".to_string(), "Configuration saved".to_string()));
        }
        
        if load_requested {
            self.load_config();
            self.messages.push(("System".to_string(), "Configuration loaded".to_string()));
        }
        
        if close {
            self.show_config_window = false;
        }
    }
    
    pub fn show_config_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("⚙️ Configuration").clicked() {
            self.show_config_window = true;
        }
    }
}