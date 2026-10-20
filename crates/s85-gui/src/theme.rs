//! Dark theme with an orange accent, in the spirit of the official app.

use eframe::egui::{self, Color32};

pub const ACCENT: Color32 = Color32::from_rgb(201, 81, 0);
pub const BG: Color32 = Color32::from_rgb(24, 24, 27);
pub const PANEL: Color32 = Color32::from_rgb(16, 16, 18);
pub const CASE: Color32 = Color32::from_rgb(232, 230, 235);
pub const KEY: Color32 = Color32::from_rgb(196, 202, 232);
pub const KEY_HOVER: Color32 = Color32::from_rgb(225, 228, 248);
pub const KEY_EDGE: Color32 = Color32::from_rgb(120, 124, 150);
pub const CHANGED: Color32 = Color32::from_rgb(230, 30, 30);
pub const TEXT: Color32 = Color32::from_rgb(30, 30, 40);
pub const DIM: Color32 = Color32::from_rgb(130, 130, 145);

pub fn apply(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = BG;
    v.extreme_bg_color = PANEL;
    v.selection.bg_fill = ACCENT;
    v.hyperlink_color = ACCENT;
    v.widgets.hovered.bg_stroke.color = ACCENT;
    v.widgets.active.bg_fill = ACCENT;
    v.slider_trailing_fill = true;
    ctx.set_visuals(v);
}
