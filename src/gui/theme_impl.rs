use eframe::egui;
use crate::gui::app::MeshtasticGuiApp;

impl MeshtasticGuiApp {
    /// Apply theme to the UI context
    pub fn apply_theme(&self, ctx: &egui::Context) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }
    }
    
    /// Toggle between dark and light mode
    pub fn toggle_theme(&mut self, ctx: &egui::Context) {
        self.dark_mode = !self.dark_mode;
        self.apply_theme(ctx);
        self.save_config();
        self.messages.push(("System".to_string(), 
            if self.dark_mode { 
                "Switched to dark mode".to_string() 
            } else { 
                "Switched to light mode".to_string() 
            }
        ));
    }
    
    /// UI for theme settings
    pub fn theme_settings_ui(&mut self, ctx: &egui::Context) {
        let mut close = false;
        let mut apply_requested = false;
        
        // Create local copy to avoid borrowing issues
        let mut dark_mode = self.dark_mode;
        
        egui::Window::new("Theme Settings")
            .resizable(false)
            .default_width(300.0)
            .open(&mut self.show_theme_settings)
            .show(ctx, |ui| {
                ui.heading("Appearance");
                ui.separator();
                
                ui.label("Select theme:");
                
                ui.horizontal(|ui| {
                    if ui.selectable_label(!dark_mode, "🌞 Light Mode").clicked() {
                        dark_mode = false;
                        apply_requested = true;
                    }
                    
                    if ui.selectable_label(dark_mode, "🌙 Dark Mode").clicked() {
                        dark_mode = true;
                        apply_requested = true;
                    }
                });
                
                ui.separator();
                
                // Preview section
                ui.label("Preview:");
                ui.add_space(10.0);
                
                // Create a small preview area
                let preview_frame = egui::Frame::default()
                    .fill(if dark_mode { 
                        egui::Color32::from_gray(30) 
                    } else { 
                        egui::Color32::from_gray(240) 
                    })
                    .inner_margin(10.0);
                
                preview_frame.show(ui, |ui| {
                    ui.label(egui::RichText::new("Sample Text")
                        .color(if dark_mode { 
                            egui::Color32::from_gray(220) 
                        } else { 
                            egui::Color32::from_gray(30) 
                        }));
                    
                    ui.add_space(5.0);
                    
                    ui.label(egui::RichText::new("This is how the interface will look")
                        .color(if dark_mode { 
                            egui::Color32::from_gray(180) 
                        } else { 
                            egui::Color32::from_gray(80) 
                        }));
                });
                
                ui.separator();
                
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        apply_requested = true;
                        close = true;
                    }
                    
                    if ui.button("Cancel").clicked() {
                        close = true;
                    }
                });
                
                ui.separator();
                ui.label("Note: Theme changes take effect immediately.");
            });
        
        // Handle apply request outside the closure
        if apply_requested {
            self.dark_mode = dark_mode;
            self.apply_theme(ctx);
            self.save_config();
        }
        
        if close {
            self.show_theme_settings = false;
        }
    }
    
    /// Show theme toggle button in the main UI
    pub fn show_theme_button(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let button_text = if self.dark_mode { "🌙" } else { "🌞" };
        let tooltip_text = if self.dark_mode { "Switch to light mode" } else { "Switch to dark mode" };
        
        if ui.button(button_text).on_hover_text(tooltip_text).clicked() {
            self.toggle_theme(ctx);
        }
        
        // Settings button
        if ui.button("⚙️").on_hover_text("Theme settings").clicked() {
            self.show_theme_settings = true;
        }
    }
    
    /// Initialize theme on app startup
    pub fn init_theme(&self, ctx: &egui::Context) {
        self.apply_theme(ctx);
    }
}