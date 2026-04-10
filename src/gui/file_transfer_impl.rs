use eframe::egui;
use crate::gui::BridgeCommand;
use crate::gui::app::MeshtasticGuiApp;

impl MeshtasticGuiApp {
    pub fn send_file(&mut self, ctx: &egui::Context) {
        if let (Some(file_path), Some(peer)) = (self.selected_file_path.as_ref(), self.selected_peer.as_ref()) {
            if let Err(e) = self.bridge_cmd_tx.send(BridgeCommand::SendFile {
                dest_hash: peer.file_hash.clone(),
                file_path: file_path.to_string(),
            }) {
                self.messages.push(("Reticulum".to_string(), format!("[File ERROR] {}", e)));
            } else {
                self.messages.push(("Reticulum".to_string(), format!("Sending file: {}", file_path)));
            }
            ctx.request_repaint();
        } else {
            self.messages.push(("Reticulum".to_string(), "No file or peer selected.".to_string()));
        }
    }

    pub fn file_transfer_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📁 Select File").clicked() {
                if let Some(path) = rfd::FileDialog::new().pick_file() {
                    self.selected_file_path = Some(path.display().to_string());
                    ctx.request_repaint();
                }
            }
            if let Some(ref file_path) = self.selected_file_path {
                ui.label(format!("File: {}", file_path));
                if ui.button("Send File").clicked() && self.selected_peer.is_some() {
                    self.send_file(ctx);
                }
            } else {
                ui.label("No file selected");
            }
        });
        if let Some((ref file_name, progress)) = self.transfer_progress {
            ui.label(format!("Sending {}: {:.0}%", file_name, progress * 100.0));
        }
    }
}