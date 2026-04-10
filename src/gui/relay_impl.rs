use eframe::egui;
use crate::gui::app::MeshtasticGuiApp;
use crate::gui::relay::RelayDirection;

impl MeshtasticGuiApp {
    pub fn relay_settings_ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.relay_enabled, "Enable Relay");
        ui.horizontal(|ui| {
            ui.label("Direction:");
            egui::ComboBox::from_label("relay_direction")
                .selected_text(format!("{:?}", self.relay_direction))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.relay_direction, RelayDirection::Both, "Both");
                    ui.selectable_value(&mut self.relay_direction, RelayDirection::MqttToReticulum, "MQTT → Reticulum");
                    ui.selectable_value(&mut self.relay_direction, RelayDirection::ReticulumToMqtt, "Reticulum → MQTT");
                });
        });
        ui.horizontal(|ui| {
            ui.label("Forward Reticulum to MQTT channel:");
            egui::ComboBox::from_label("relay_target_channel")
                .selected_text(&self.relay_target_channel)
                .show_ui(ui, |ui| {
                    for ch in &self.channels {
                        ui.selectable_value(&mut self.relay_target_channel, ch.clone(), ch);
                    }
                });
        });
        if let Some(peer) = &self.selected_peer {
            ui.label(format!("Forward MQTT to Reticulum peer: {}…", &peer.main_hash[..16]));
        } else {
            ui.label("No peer selected for MQTT → Reticulum forwarding.");
        }
    }
}