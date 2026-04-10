//! GUI visualization utilities for signal strength and network metrics
//!
//! This module provides utilities for displaying signal strength indicators,
//! network quality bars, and other visualizations in the GUI.

use eframe::egui;
use egui::{Color32, Stroke, Rect, Pos2};
use std::f32::consts::PI;

/// Signal strength visualization styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalStyle {
    /// Classic cell phone signal bars
    Bars,
    /// Circular signal strength indicator
    Circular,
    /// Simple text indicator
    Text,
    /// Combined bars with text
    BarsWithText,
}

/// Signal strength level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SignalLevel {
    Excellent,
    Good,
    Fair,
    Poor,
    None,
}

impl SignalLevel {
    /// Convert RSSI value to signal level
    pub fn from_rssi(rssi: i32) -> Self {
        if rssi >= -70 {
            SignalLevel::Excellent
        } else if rssi >= -85 {
            SignalLevel::Good
        } else if rssi >= -100 {
            SignalLevel::Fair
        } else if rssi >= -115 {
            SignalLevel::Poor
        } else {
            SignalLevel::None
        }
    }
    
    /// Convert link quality percentage to signal level
    pub fn from_link_quality(quality: u8) -> Self {
        if quality >= 80 {
            SignalLevel::Excellent
        } else if quality >= 60 {
            SignalLevel::Good
        } else if quality >= 40 {
            SignalLevel::Fair
        } else if quality >= 20 {
            SignalLevel::Poor
        } else {
            SignalLevel::None
        }
    }
    
    /// Get color for this signal level
    pub fn color(&self) -> Color32 {
        match self {
            SignalLevel::Excellent => Color32::from_rgb(0, 200, 0),   // Green
            SignalLevel::Good => Color32::from_rgb(150, 200, 0),      // Yellow-green
            SignalLevel::Fair => Color32::from_rgb(255, 200, 0),      // Yellow
            SignalLevel::Poor => Color32::from_rgb(255, 100, 0),      // Orange
            SignalLevel::None => Color32::from_rgb(255, 50, 50),      // Red
        }
    }
    
    /// Get text representation
    pub fn text(&self) -> &'static str {
        match self {
            SignalLevel::Excellent => "Excellent",
            SignalLevel::Good => "Good",
            SignalLevel::Fair => "Fair",
            SignalLevel::Poor => "Poor",
            SignalLevel::None => "None",
        }
    }
}

/// Signal strength visualizer
pub struct SignalVisualizer {
    style: SignalStyle,
    size: f32,
    show_text: bool,
    show_tooltip: bool,
}

impl SignalVisualizer {
    /// Create a new signal visualizer
    pub fn new() -> Self {
        SignalVisualizer {
            style: SignalStyle::Bars,
            size: 16.0,
            show_text: false,
            show_tooltip: true,
        }
    }
    
    /// Set visualization style
    pub fn with_style(mut self, style: SignalStyle) -> Self {
        self.style = style;
        self
    }
    
    /// Set size
    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
    
    /// Show text label
    pub fn with_text(mut self, show: bool) -> Self {
        self.show_text = show;
        self
    }
    
    /// Show tooltip
    pub fn with_tooltip(mut self, show: bool) -> Self {
        self.show_tooltip = show;
        self
    }
    
    /// Draw signal strength indicator
    pub fn draw(&self, ui: &mut egui::Ui, rssi: Option<i32>, link_quality: Option<u8>) -> egui::Response {
        let signal_level = if let Some(rssi) = rssi {
            SignalLevel::from_rssi(rssi)
        } else if let Some(quality) = link_quality {
            SignalLevel::from_link_quality(quality)
        } else {
            SignalLevel::None
        };
        
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(self.size, self.size),
            egui::Sense::hover()
        );
        
        let rect = response.rect;
        let center = rect.center();
        
        // Draw the signal indicator
        match self.style {
            SignalStyle::Bars => self.draw_bars(&painter, rect, signal_level),
            SignalStyle::Circular => self.draw_circular(&painter, center, signal_level),
            SignalStyle::Text => self.draw_text(&painter, rect, signal_level, rssi, link_quality),
            SignalStyle::BarsWithText => {
                self.draw_bars(&painter, rect, signal_level);
                self.draw_text(&painter, rect, signal_level, rssi, link_quality);
            }
        }
        
        // Add tooltip if enabled
        if self.show_tooltip && response.hovered() {
            let tooltip_text = self.create_tooltip_text(rssi, link_quality, signal_level);
            response.clone().on_hover_text(tooltip_text);
        }
        
        response
    }
    
    /// Draw classic signal bars
    fn draw_bars(&self, painter: &egui::Painter, rect: Rect, signal_level: SignalLevel) {
        let bar_count = 4;
        let bar_width = rect.width() / (bar_count as f32 * 2.0);
        let bar_spacing = bar_width * 0.5;
        let max_bar_height = rect.height();
        
        let start_x = rect.left();
        let bottom_y = rect.bottom();
        
        for i in 0..bar_count {
            let bar_index = bar_count - i - 1;
            let bar_height = max_bar_height * (i as f32 + 1.0) / bar_count as f32;
            
            let x = start_x + (bar_width + bar_spacing) * bar_index as f32;
            let bar_rect = Rect::from_min_max(
                Pos2::new(x, bottom_y - bar_height),
                Pos2::new(x + bar_width, bottom_y)
            );
            
            // Determine if this bar should be filled based on signal level
            let fill_bar = match signal_level {
                SignalLevel::Excellent => i >= 0, // All bars
                SignalLevel::Good => i >= 1,      // 3 bars
                SignalLevel::Fair => i >= 2,      // 2 bars
                SignalLevel::Poor => i >= 3,      // 1 bar
                SignalLevel::None => false,       // No bars
            };
            
            let color = if fill_bar {
                signal_level.color()
            } else {
                Color32::from_gray(100) // Gray for unfilled bars
            };
            
            painter.rect_filled(bar_rect, 2.0, color);
        }
    }
    
    /// Draw circular signal indicator
    fn draw_circular(&self, painter: &egui::Painter, center: Pos2, signal_level: SignalLevel) {
        let radius = self.size / 2.0;
        let stroke_width = radius / 4.0;
        
        // Draw background circle
        painter.circle_stroke(
            center,
            radius,
            Stroke::new(stroke_width, Color32::from_gray(100))
        );
        
        // Draw signal arc based on level
        let arc_angle = match signal_level {
            SignalLevel::Excellent => 2.0 * PI,
            SignalLevel::Good => 1.5 * PI,
            SignalLevel::Fair => PI,
            SignalLevel::Poor => 0.5 * PI,
            SignalLevel::None => 0.0,
        };
        
        if arc_angle > 0.0 {
            // Draw filled arc using circle segment
            // Simplified approach: draw a filled circle segment
            let points = (0..=20).map(|i| {
                let angle = (i as f32 / 20.0) * arc_angle;
                Pos2::new(
                    center.x + (radius - stroke_width / 2.0) * angle.cos(),
                    center.y + (radius - stroke_width / 2.0) * angle.sin(),
                )
            }).collect::<Vec<_>>();
            
            if !points.is_empty() {
                painter.add(egui::Shape::convex_polygon(
                    points,
                    signal_level.color(),
                    Stroke::NONE,
                ));
            }
        }
        
        // Draw center dot
        painter.circle_filled(center, radius / 4.0, Color32::from_gray(50));
    }
    
    /// Draw text indicator
    fn draw_text(&self, painter: &egui::Painter, rect: Rect, signal_level: SignalLevel, 
                 rssi: Option<i32>, link_quality: Option<u8>) {
        let text = if self.show_text {
            if let Some(rssi) = rssi {
                format!("{} dBm", rssi)
            } else if let Some(quality) = link_quality {
                format!("{}%", quality)
            } else {
                signal_level.text().to_string()
            }
        } else {
            signal_level.text().to_string()
        };
        
        let color = signal_level.color();
        let galley = painter.layout(
            text,
            egui::FontId::monospace(self.size * 0.6),
            color,
            rect.width()
        );
        
        let text_pos = rect.center() - galley.size() / 2.0;
        painter.galley(text_pos, galley, color);
    }
    
    /// Create tooltip text
    fn create_tooltip_text(&self, rssi: Option<i32>, link_quality: Option<u8>, signal_level: SignalLevel) -> String {
        let mut parts = Vec::new();
        
        if let Some(rssi) = rssi {
            parts.push(format!("RSSI: {} dBm", rssi));
        }
        
        if let Some(quality) = link_quality {
            parts.push(format!("Link Quality: {}%", quality));
        }
        
        parts.push(format!("Signal: {}", signal_level.text()));
        
        parts.join("\n")
    }
}

/// Network metrics visualization
pub struct NetworkMetricsVisualizer {
    show_latency: bool,
    show_packet_loss: bool,
    show_quality: bool,
    compact: bool,
}

impl NetworkMetricsVisualizer {
    /// Create a new network metrics visualizer
    pub fn new() -> Self {
        NetworkMetricsVisualizer {
            show_latency: true,
            show_packet_loss: true,
            show_quality: true,
            compact: false,
        }
    }
    
    /// Show latency indicator
    pub fn with_latency(mut self, show: bool) -> Self {
        self.show_latency = show;
        self
    }
    
    /// Show packet loss indicator
    pub fn with_packet_loss(mut self, show: bool) -> Self {
        self.show_packet_loss = show;
        self
    }
    
    /// Show quality indicator
    pub fn with_quality(mut self, show: bool) -> Self {
        self.show_quality = show;
        self
    }
    
    /// Use compact layout
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
    
    /// Draw network metrics
    pub fn draw(&self, ui: &mut egui::Ui, latency_ms: Option<f32>, 
                packet_loss_percent: Option<f32>, quality_score: Option<u8>) {
        if self.compact {
            self.draw_compact(ui, latency_ms, packet_loss_percent, quality_score);
        } else {
            self.draw_detailed(ui, latency_ms, packet_loss_percent, quality_score);
        }
    }
    
    /// Draw detailed metrics
    fn draw_detailed(&self, ui: &mut egui::Ui, latency_ms: Option<f32>, 
                     packet_loss_percent: Option<f32>, quality_score: Option<u8>) {
        ui.vertical(|ui| {
            if self.show_latency {
                if let Some(latency) = latency_ms {
                    ui.horizontal(|ui| {
                        ui.label("Latency:");
                        self.draw_latency_bar(ui, latency);
                        ui.label(format!("{:.1} ms", latency));
                    });
                }
            }
            
            if self.show_packet_loss {
                if let Some(packet_loss) = packet_loss_percent {
                    ui.horizontal(|ui| {
                        ui.label("Packet Loss:");
                        self.draw_packet_loss_bar(ui, packet_loss);
                        ui.label(format!("{:.1}%", packet_loss));
                    });
                }
            }
            
            if self.show_quality {
                if let Some(quality) = quality_score {
                    ui.horizontal(|ui| {
                        ui.label("Quality:");
                        self.draw_quality_bar(ui, quality);
                        ui.label(format!("{}%", quality));
                    });
                }
            }
        });
    }
    
    /// Draw compact metrics
    fn draw_compact(&self, ui: &mut egui::Ui, latency_ms: Option<f32>, 
                    packet_loss_percent: Option<f32>, quality_score: Option<u8>) {
        ui.horizontal(|ui| {
            if self.show_latency {
                if let Some(latency) = latency_ms {
                    ui.label(format!("⏱{:.0}ms", latency));
                }
            }
            
            if self.show_packet_loss {
                if let Some(packet_loss) = packet_loss_percent {
                    ui.label(format!("📦{:.0}%", packet_loss));
                }
            }
            
            if self.show_quality {
                if let Some(quality) = quality_score {
                    let (emoji, color) = self.get_quality_emoji(quality);
                    ui.colored_label(color, format!("{}{}%", emoji, quality));
                }
            }
        });
    }
    
    /// Draw latency bar
    fn draw_latency_bar(&self, ui: &mut egui::Ui, latency_ms: f32) {
        let width = 100.0;
        let height = 8.0;
        
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(width, height),
            egui::Sense::hover()
        );
        
        let rect = response.rect;
        
        // Draw background
        painter.rect_filled(rect, 2.0, Color32::from_gray(50));
        
        // Calculate fill percentage (lower latency = better)
        let fill_percent = (500.0 - latency_ms.min(500.0)) / 500.0;
        let fill_width = rect.width() * fill_percent.max(0.0);
        
        if fill_width > 0.0 {
            let fill_rect = Rect::from_min_max(
                rect.min,
                Pos2::new(rect.min.x + fill_width, rect.max.y)
            );
            
            let color = self.get_latency_color(latency_ms);
            painter.rect_filled(fill_rect, 2.0, color);
        }
        
        if response.hovered() {
            response.clone().on_hover_text(format!("Latency: {:.1} ms", latency_ms));
        }
    }
    
    /// Draw packet loss bar
    fn draw_packet_loss_bar(&self, ui: &mut egui::Ui, packet_loss_percent: f32) {
        let width = 100.0;
        let height = 8.0;
        
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(width, height),
            egui::Sense::hover()
        );
        
        let rect = response.rect;
        
        // Draw background
        painter.rect_filled(rect, 2.0, Color32::from_gray(50));
        
        // Calculate fill percentage (lower packet loss = better)
        let fill_percent = 1.0 - (packet_loss_percent.min(100.0) / 100.0);
        let fill_width = rect.width() * fill_percent.max(0.0);
        
        if fill_width > 0.0 {
            let fill_rect = Rect::from_min_max(
                rect.min,
                Pos2::new(rect.min.x + fill_width, rect.max.y)
            );
            
            let color = self.get_packet_loss_color(packet_loss_percent);
            painter.rect_filled(fill_rect, 2.0, color);
        }
        
        if response.hovered() {
            response.clone().on_hover_text(format!("Packet Loss: {:.1}%", packet_loss_percent));
        }
    }
    
    /// Draw quality bar
    fn draw_quality_bar(&self, ui: &mut egui::Ui, quality_score: u8) {
        let width = 100.0;
        let height = 8.0;
        
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(width, height),
            egui::Sense::hover()
        );
        
        let rect = response.rect;
        
        // Draw background
        painter.rect_filled(rect, 2.0, Color32::from_gray(50));
        
        // Calculate fill percentage
        let fill_percent = quality_score as f32 / 100.0;
        let fill_width = rect.width() * fill_percent;
        
        if fill_width > 0.0 {
            let fill_rect = Rect::from_min_max(
                rect.min,
                Pos2::new(rect.min.x + fill_width, rect.max.y)
            );
            
            let color = self.get_quality_color(quality_score);
            painter.rect_filled(fill_rect, 2.0, color);
        }
        
        if response.hovered() {
            response.clone().on_hover_text(format!("Quality Score: {}%", quality_score));
        }
    }
    
    /// Get color for latency value
    fn get_latency_color(&self, latency_ms: f32) -> Color32 {
        if latency_ms < 50.0 {
            Color32::from_rgb(0, 200, 0)     // Green
        } else if latency_ms < 100.0 {
            Color32::from_rgb(150, 200, 0)   // Yellow-green
        } else if latency_ms < 200.0 {
            Color32::from_rgb(255, 200, 0)   // Yellow
        } else if latency_ms < 500.0 {
            Color32::from_rgb(255, 100, 0)   // Orange
        } else {
            Color32::from_rgb(255, 50, 50)   // Red
        }
    }
    
    /// Get color for packet loss value
    fn get_packet_loss_color(&self, packet_loss_percent: f32) -> Color32 {
        if packet_loss_percent < 1.0 {
            Color32::from_rgb(0, 200, 0)     // Green
        } else if packet_loss_percent < 5.0 {
            Color32::from_rgb(150, 200, 0)   // Yellow-green
        } else if packet_loss_percent < 10.0 {
            Color32::from_rgb(255, 200, 0)   // Yellow
        } else if packet_loss_percent < 20.0 {
            Color32::from_rgb(255, 100, 0)   // Orange
        } else {
            Color32::from_rgb(255, 50, 50)   // Red
        }
    }
    
    /// Get color for quality score
    fn get_quality_color(&self, quality_score: u8) -> Color32 {
        if quality_score >= 80 {
            Color32::from_rgb(0, 200, 0)     // Green
        } else if quality_score >= 60 {
            Color32::from_rgb(150, 200, 0)   // Yellow-green
        } else if quality_score >= 40 {
            Color32::from_rgb(255, 200, 0)   // Yellow
        } else if quality_score >= 20 {
            Color32::from_rgb(255, 100, 0)   // Orange
        } else {
            Color32::from_rgb(255, 50, 50)   // Red
        }
    }
    
    /// Get emoji for quality score
    fn get_quality_emoji(&self, quality_score: u8) -> (&'static str, Color32) {
        if quality_score >= 80 {
            ("⭐", Color32::from_rgb(255, 215, 0))    // Gold star
        } else if quality_score >= 60 {
            ("👍", Color32::from_rgb(0, 200, 0))      // Green thumb
        } else if quality_score >= 40 {
            ("😐", Color32::from_rgb(255, 200, 0))    // Yellow neutral
        } else if quality_score >= 20 {
            ("👎", Color32::from_rgb(255, 100, 0))    // Orange thumb down
        } else {
            ("💀", Color32::from_rgb(255, 50, 50))    // Red skull
        }
    }
}

/// Interface type visualizer
pub struct InterfaceVisualizer {
    show_icon: bool,
    show_name: bool,
    compact: bool,
}

impl InterfaceVisualizer {
    /// Create a new interface visualizer
    pub fn new() -> Self {
        InterfaceVisualizer {
            show_icon: true,
            show_name: true,
            compact: false,
        }
    }
    
    /// Show icon
    pub fn with_icon(mut self, show: bool) -> Self {
        self.show_icon = show;
        self
    }
    
    /// Show name
    pub fn with_name(mut self, show: bool) -> Self {
        self.show_name = show;
        self
    }
    
    /// Use compact layout
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
    
    /// Draw interface indicator
    pub fn draw(&self, ui: &mut egui::Ui, interface_type: &str) {
        let (icon, color) = self.get_interface_icon(interface_type);
        
        if self.compact {
            ui.horizontal(|ui| {
                if self.show_icon {
                    ui.colored_label(color, icon);
                }
                if self.show_name {
                    ui.label(interface_type);
                }
            });
        } else {
            ui.vertical(|ui| {
                if self.show_icon {
                    ui.heading(icon);
                }
                if self.show_name {
                    ui.label(interface_type);
                }
            });
        }
    }
    
    /// Get icon for interface type
    fn get_interface_icon(&self, interface_type: &str) -> (&'static str, Color32) {
        if interface_type.contains("LoRa") || interface_type.contains("Radio") {
            ("📡", Color32::from_rgb(0, 150, 200))    // Blue satellite
        } else if interface_type.contains("TCP") || interface_type.contains("UDP") {
            ("🌐", Color32::from_rgb(0, 200, 150))    // Teal globe
        } else if interface_type.contains("Serial") {
            ("🔌", Color32::from_rgb(200, 150, 0))    // Orange plug
        } else if interface_type.contains("MQTT") {
            ("📨", Color32::from_rgb(150, 0, 200))    // Purple envelope
        } else if interface_type.contains("I2P") {
            ("🕵️", Color32::from_rgb(100, 100, 100)) // Gray spy
        } else {
            ("❓", Color32::from_gray(150))           // Gray question
        }
    }
}