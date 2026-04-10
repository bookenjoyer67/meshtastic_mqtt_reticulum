use eframe::egui;
use crate::gui::GuiToMqtt;
use prost::Message;
use base64::engine::Engine as _;
use qrcode::QrCode;
use image::{Luma, ImageBuffer};
use crate::gui::app::MeshtasticGuiApp;

impl MeshtasticGuiApp {
    pub fn join_channel(&mut self, ctx: &egui::Context) {
        let name = self.new_channel_name.trim().to_string();
        if name.is_empty() {
            self.messages.push(("System".to_string(), "Channel name is required".to_string()));
            return;
        }
        let url = self.channel_url_input.trim().to_string();
        let mut psk = self.new_channel_psk.trim().to_string();
        if !url.is_empty() {
            if let Some(url_psk) = self.get_psk_from_url(&url) {
                psk = url_psk;
                self.messages.push(("System".to_string(), format!("Using PSK from URL for channel '{}'", name)));
            } else {
                self.messages.push(("System".to_string(), "Invalid URL – no valid PSK found. Using default or manual PSK.".to_string()));
            }
        }
        if psk.is_empty() {
            psk = crate::encryption::generate_random_psk();
            self.messages.push(("System".to_string(), format!("Generated secure random PSK for channel '{}'", name)));
        } else if psk == "AQ==" || psk.len() < 16 {
            self.messages.push(("System".to_string(), "Warning: PSK is weak or too short. Consider using a longer, cryptographically secure PSK.".to_string()));
        }
        let _ = self.mqtt_cmd_tx.send(GuiToMqtt::AddChannel { name: name.clone(), psk: psk.clone() });
        self.channels.push(name.clone());
        self.channel_psks.insert(name.clone(), psk);
        self.messages.push(("System".to_string(), format!("Channel '{}' added", name)));
        self.new_channel_name.clear();
        self.new_channel_psk.clear();
        self.channel_url_input.clear();
        ctx.request_repaint();
    }

    fn get_psk_from_url(&self, url: &str) -> Option<String> {
        let fragment = url.split('#').nth(1)?;
        let cleaned: String = fragment.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect();
        if cleaned.is_empty() { return None; }
        let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&cleaned).ok()?;
        use meshtastic_protobufs::meshtastic::{Channel, ChannelSet, ChannelSettings};
        if let Ok(channel_set) = ChannelSet::decode(&decoded[..]) {
            if let Some(settings) = channel_set.settings.first() {
                if !settings.psk.is_empty() {
                    return Some(base64::engine::general_purpose::STANDARD.encode(&settings.psk));
                }
            }
        }
        if let Ok(channel) = Channel::decode(&decoded[..]) {
            if let Some(settings) = channel.settings {
                if !settings.psk.is_empty() {
                    return Some(base64::engine::general_purpose::STANDARD.encode(&settings.psk));
                }
            }
        }
        if let Ok(settings) = ChannelSettings::decode(&decoded[..]) {
            if !settings.psk.is_empty() {
                return Some(base64::engine::general_purpose::STANDARD.encode(&settings.psk));
            }
        }
        None
    }

    pub fn remove_channel(&mut self, ctx: &egui::Context, name: &str) {
        let _ = self.mqtt_cmd_tx.send(GuiToMqtt::RemoveChannel { name: name.to_string() });
        self.channels.retain(|c| c != name);
        self.channel_psks.remove(name);
        if self.active_channel == name {
            self.active_channel = self.channels.first().cloned().unwrap_or_default();
        }
        ctx.request_repaint();
    }

    fn channel_to_share_url(&self, name: &str, psk_base64: &str) -> Option<String> {
        let psk_bytes = base64::engine::general_purpose::STANDARD.decode(psk_base64).ok()?;
        let settings = meshtastic_protobufs::meshtastic::ChannelSettings {
            name: name.to_string(),
            psk: psk_bytes,
            ..Default::default()
        };
        let channel = meshtastic_protobufs::meshtastic::Channel {
            settings: Some(settings),
            ..Default::default()
        };
        let bytes = channel.encode_to_vec();
        let fragment = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes);
        Some(format!("https://meshtastic.org/v/#{}", fragment))
    }

    fn generate_qr_texture(&mut self, ctx: &egui::Context, share_url: &str) -> Option<egui::TextureHandle> {
        let code = QrCode::new(share_url.as_bytes()).ok()?;
        let size = code.width();
        let mut img = ImageBuffer::new(size as u32, size as u32);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let dark = code[(x as usize, y as usize)] == qrcode::Color::Dark;
            *pixel = Luma([if dark { 0u8 } else { 255u8 }]);
        }
        let img = image::DynamicImage::ImageLuma8(img);
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba.into_raw());
        Some(ctx.load_texture("qr_code", color_image, egui::TextureOptions::LINEAR))
    }

    pub fn show_qr_for_channel(&mut self, ctx: &egui::Context, channel_name: &str) {
        if let Some(psk) = self.channel_psks.get(channel_name) {
            if let Some(share_url) = self.channel_to_share_url(channel_name, psk) {
                if let Some(texture) = self.generate_qr_texture(ctx, &share_url) {
                    self.qr_texture = Some(texture);
                    self.current_qr_channel = Some(channel_name.to_string());
                    self.show_qr_window = true;
                } else {
                    self.messages.push(("System".to_string(), "Failed to generate QR code".to_string()));
                }
            } else {
                self.messages.push(("System".to_string(), "Failed to create share URL".to_string()));
            }
        } else {
            self.messages.push(("System".to_string(), format!("No PSK stored for channel '{}'", channel_name)));
        }
    }

    pub fn channel_management_ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Channel name:");
            ui.text_edit_singleline(&mut self.new_channel_name);
        });
        ui.horizontal(|ui| {
            ui.label("Channel URL:");
            ui.text_edit_singleline(&mut self.channel_url_input);
        });
        ui.horizontal(|ui| {
            ui.label("PSK:");
            ui.text_edit_singleline(&mut self.new_channel_psk);
        });
        if ui.button("Join").clicked() {
            self.join_channel(ctx);
        }
        ui.separator();
        ui.label("Active channels:");
        let mut to_remove = None;
        let mut qr_channel = None;
        for ch in &self.channels {
            let ch_clone = ch.clone();
            ui.horizontal(|ui| {
                if ui.button("❌").clicked() {
                    to_remove = Some(ch_clone);
                }
                if ui.button("📱 QR").clicked() {
                    qr_channel = Some(ch.clone());
                }
                ui.label(ch);
            });
        }
        if let Some(ch_name) = to_remove {
            self.remove_channel(ctx, &ch_name);
        }
        if let Some(ch_name) = qr_channel {
            self.show_qr_for_channel(ctx, &ch_name);
        }
        ui.separator();
        ui.label("ℹ️ Channel name is always required. Provide a URL to automatically set the PSK (otherwise use manual PSK).");
    }
}