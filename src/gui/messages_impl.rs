use eframe::egui;
use crate::gui::GuiToMqtt;
use crate::gui::BridgeCommand;
use crate::gui::app::MeshtasticGuiApp;

impl MeshtasticGuiApp {
    pub fn send_message(&mut self, ctx: &egui::Context) {
        let text = self.input_text.trim().to_string();
        if text.is_empty() {
            return;
        }
        let channel = self.active_channel.clone();
        // MQTT
        if let Err(e) = self.mqtt_cmd_tx.send(GuiToMqtt::SendMessage { channel: channel.clone(), text: text.clone() }) {
            self.messages.push((channel, format!("[MQTT ERROR] {}", e)));
        } else {
            self.messages.push((channel, format!("You: {}", text)));
        }
        // Reticulum
        if let Some(peer) = &self.selected_peer {
            let formatted_text = format!("[{}] {}", self.nickname, text);
            if let Err(e) = self.bridge_cmd_tx.send(BridgeCommand::SendMessage {
                dest_hash: peer.main_hash.clone(),
                text: formatted_text,
            }) {
                self.messages.push(("Reticulum".to_string(), format!("[ERROR] {}", e)));
            }
        } else if !self.peers.is_empty() {
            self.messages.push(("Reticulum".to_string(), "No peer selected. Click a peer to send.".to_string()));
        }
        self.input_text.clear();
        ctx.request_repaint();
    }

    pub fn chat_area(&mut self, ui: &mut egui::Ui) {
        let available_height = ui.available_height() - 60.0;
        let max_height = available_height.max(100.0);
        egui::ScrollArea::vertical()
            .max_height(max_height)
            .stick_to_bottom(true)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for (source, text) in &self.messages {
                    ui.label(format!("[{}] {}", source, text));
                }
            });
    }

    pub fn message_input_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let text_edit = ui.text_edit_singleline(&mut self.input_text);
            if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.send_message(ctx);
            }
            if ui.button("Send").clicked() {
                self.send_message(ctx);
            }
        });
    }
}