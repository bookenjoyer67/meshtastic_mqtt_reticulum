use eframe::egui;
use crate::gui::app::MeshtasticGuiApp;

impl MeshtasticGuiApp {
    pub fn node_list_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("node_list")
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Meshtastic Nodes");
                ui.separator();
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
                if self.nodes.is_empty() {
                    ui.label("No nodes discovered yet.");
                    ui.label("(NodeInfo/Position packets will appear here)");
                }
            });
    }
}