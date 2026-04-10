use eframe::egui;
use crate::gui::app::{MeshtasticGuiApp, MessageFilterSource};

impl MeshtasticGuiApp {
    /// Filter messages based on search text and source filter
    pub fn filtered_messages(&self) -> Vec<(String, String)> {
        let search_text_lower = self.message_search_text.to_lowercase();
        
        self.messages.iter()
            .filter(|(source, text)| {
                // Apply source filter
                let source_matches = match &self.message_filter_source {
                    MessageFilterSource::All => true,
                    MessageFilterSource::Mqtt => source == "MQTT" || source.contains("MQTT"),
                    MessageFilterSource::Reticulum => source == "Reticulum" || source.contains("Reticulum"),
                    MessageFilterSource::System => source == "System",
                    MessageFilterSource::Custom(custom_source) => source == custom_source,
                };
                
                if !source_matches {
                    return false;
                }
                
                // Apply text search if search text is not empty
                if !search_text_lower.is_empty() {
                    let text_lower = text.to_lowercase();
                    let source_lower = source.to_lowercase();
                    text_lower.contains(&search_text_lower) || source_lower.contains(&search_text_lower)
                } else {
                    true
                }
            })
            .cloned()
            .collect()
    }
    
    /// Get count of filtered messages
    pub fn filtered_message_count(&self) -> usize {
        self.filtered_messages().len()
    }
    
    /// Get total message count
    pub fn total_message_count(&self) -> usize {
        self.messages.len()
    }
    
    /// Clear search filters
    pub fn clear_search(&mut self) {
        self.message_search_text.clear();
        self.message_filter_source = MessageFilterSource::All;
    }
    
    /// UI for message search and filtering
    pub fn search_panel_ui(&mut self, ctx: &egui::Context) {
        let mut close = false;
        let mut clear_requested = false;
        
        // Get filtered count before entering closure to avoid borrowing issues
        let filtered_count = self.filtered_message_count();
        let total_count = self.total_message_count();
        
        egui::Window::new("Message Search & Filter")
            .resizable(true)
            .default_width(500.0)
            .open(&mut self.show_search_panel)
            .show(ctx, |ui| {
                ui.heading("Search Messages");
                ui.separator();
                
                // Search text input
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.message_search_text);
                    
                    if !self.message_search_text.is_empty() && ui.button("✕").clicked() {
                        self.message_search_text.clear();
                    }
                });
                
                ui.separator();
                
                // Source filter
                ui.label("Filter by source:");
                ui.horizontal(|ui| {
                    if ui.selectable_label(
                        matches!(self.message_filter_source, MessageFilterSource::All),
                        "All"
                    ).clicked() {
                        self.message_filter_source = MessageFilterSource::All;
                    }
                    
                    if ui.selectable_label(
                        matches!(self.message_filter_source, MessageFilterSource::Mqtt),
                        "MQTT"
                    ).clicked() {
                        self.message_filter_source = MessageFilterSource::Mqtt;
                    }
                    
                    if ui.selectable_label(
                        matches!(self.message_filter_source, MessageFilterSource::Reticulum),
                        "Reticulum"
                    ).clicked() {
                        self.message_filter_source = MessageFilterSource::Reticulum;
                    }
                    
                    if ui.selectable_label(
                        matches!(self.message_filter_source, MessageFilterSource::System),
                        "System"
                    ).clicked() {
                        self.message_filter_source = MessageFilterSource::System;
                    }
                });
                
                // Custom source filter
                ui.horizontal(|ui| {
                    ui.label("Custom source:");
                    let mut custom_source = String::new();
                    if let MessageFilterSource::Custom(source) = &self.message_filter_source {
                        custom_source = source.clone();
                    }
                    
                    let response = ui.text_edit_singleline(&mut custom_source);
                    if response.changed() && !custom_source.is_empty() {
                        self.message_filter_source = MessageFilterSource::Custom(custom_source.clone());
                    }
                    
                    if !custom_source.is_empty() && ui.button("Set").clicked() {
                        self.message_filter_source = MessageFilterSource::Custom(custom_source.clone());
                    }
                });
                
                ui.separator();
                
                // Statistics (using pre-calculated values)
                ui.label(format!("Showing {} of {} messages", filtered_count, total_count));
                
                if filtered_count < total_count {
                    ui.label(format!("Filtered out: {} messages", total_count - filtered_count));
                }
                
                ui.separator();
                
                // Actions
                ui.horizontal(|ui| {
                    if ui.button("Clear Filters").clicked() {
                        clear_requested = true;
                    }
                    
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            });
        
        if clear_requested {
            self.clear_search();
        }
        
        if close {
            self.show_search_panel = false;
        }
    }
    
    /// Show search button in the main UI
    pub fn show_search_button(&mut self, ui: &mut egui::Ui) {
        if ui.button("🔍 Search").clicked() {
            self.show_search_panel = true;
        }
        
        // Show search status indicator
        let has_search = !self.message_search_text.is_empty() || 
                        !matches!(self.message_filter_source, MessageFilterSource::All);
        
        if has_search {
            ui.label("(Filtered)");
        }
    }
    
    /// Update chat_area to use filtered messages
    pub fn chat_area_filtered(&mut self, ui: &mut egui::Ui) {
        let available_height = ui.available_height() - 60.0;
        let max_height = available_height.max(100.0);
        
        let filtered_messages = self.filtered_messages();
        
        egui::ScrollArea::vertical()
            .max_height(max_height)
            .stick_to_bottom(true)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for (source, text) in &filtered_messages {
                    ui.label(format!("[{}] {}", source, text));
                }
                
                // Show message if filtered messages is empty
                if filtered_messages.is_empty() && !self.messages.is_empty() {
                    ui.label("No messages match the current filters.");
                    ui.label("Try adjusting your search or filter settings.");
                }
            });
    }
}