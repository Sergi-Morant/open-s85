//! Draws the S85 from its key geometry and reports clicks.

use eframe::egui::{self, Color32, CornerRadius, FontId, Rect, Sense, Stroke, StrokeKind, Vec2};
use s85_protocol::layout::{KeyDef, KEYS};

use crate::theme;

/// Canvas the key rectangles are expressed in.
const CANVAS: Vec2 = Vec2::new(780.0, 300.0);

pub struct KeyLook {
    pub text: String,
    /// Draw the text dimmed (e.g. an empty FN-layer key showing its base legend).
    pub dim: bool,
    /// Outline in the "has a custom function" colour.
    pub changed: bool,
    /// Fill colour override (per-key lighting).
    pub fill: Option<Color32>,
}

/// Draw the keyboard; returns the matrix slot of a clicked key.
pub fn show(ui: &mut egui::Ui, selected: &[u8], look: impl Fn(&KeyDef) -> KeyLook) -> Option<u8> {
    let width = ui.available_width().min(1000.0);
    let scale = width / CANVAS.x;
    let (resp, painter) = ui.allocate_painter(CANVAS * scale, Sense::click());
    let origin = resp.rect.min;
    let map = |x: u16, y: u16| origin + Vec2::new(x as f32, y as f32) * scale;

    painter.rect_filled(resp.rect, CornerRadius::same(14), theme::CASE);
    painter.rect_stroke(resp.rect, CornerRadius::same(14), Stroke::new(1.0, theme::BORDER), StrokeKind::Inside);
    // Screen and knob area, top right, as on the real board.
    let screen = Rect::from_min_max(map(648, 22), map(712, 56));
    painter.rect_filled(screen, CornerRadius::same(4), Color32::from_rgb(8, 8, 10));
    painter.text(
        screen.center(),
        egui::Align2::CENTER_CENTER,
        crate::icons::MONITOR,
        FontId::proportional(14.0 * scale),
        theme::DIM,
    );

    let mut clicked = None;
    let pointer = resp.interact_pointer_pos().filter(|_| resp.clicked());
    let hover = resp.hover_pos();
    if hover.is_some_and(|p| {
        KEYS.iter().any(|k| Rect::from_min_max(map(k.rect.0, k.rect.1), map(k.rect.2, k.rect.3)).contains(p))
    }) {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    for k in KEYS {
        let (x1, y1, x2, y2) = k.rect;
        let r = Rect::from_min_max(map(x1, y1), map(x2, y2));
        let l = look(k);
        let is_sel = selected.contains(&k.slot);
        let hovered = hover.is_some_and(|p| r.contains(p));
        let fill = l.fill.unwrap_or(if is_sel {
            theme::ACCENT
        } else if hovered {
            theme::KEY_HOVER
        } else {
            theme::KEY
        });
        let round = if k.label == "Wheel" { CornerRadius::same(255) } else { CornerRadius::same(4) };
        painter.rect_filled(r, round, fill);
        let stroke = if l.changed { Stroke::new(2.0, theme::CHANGED) } else { Stroke::new(1.0, theme::KEY_EDGE) };
        painter.rect_stroke(r, round, stroke, StrokeKind::Inside);
        let size = (if l.text.chars().count() > 4 { 8.0 } else { 10.0 }) * scale.max(0.8);
        let color = if is_sel {
            Color32::WHITE
        } else if let Some(f) = l.fill {
            // Dark legend on bright backlight colours, light legend on dark ones.
            let luma = 0.299 * f.r() as f32 + 0.587 * f.g() as f32 + 0.114 * f.b() as f32;
            if luma > 140.0 {
                Color32::from_rgb(20, 20, 24)
            } else {
                Color32::WHITE
            }
        } else if l.dim {
            theme::DIM
        } else {
            theme::TEXT
        };
        painter.text(r.center(), egui::Align2::CENTER_CENTER, &l.text, FontId::proportional(size), color);
        if pointer.is_some_and(|p| r.contains(p)) {
            clicked = Some(k.slot);
        }
    }
    clicked
}

/// Short legend for small key caps.
pub fn short(name: &str) -> String {
    let n = match name {
        "Backspace" => "Bksp",
        "CapsLock" => "Caps",
        "PrintScreen" | "PrtSc" => "PrtSc",
        "PageUp" | "PgUp" => "PgUp",
        "PageDown" | "PgDn" => "PgDn",
        "Delete" => "Del",
        "Insert" => "Ins",
        "Wheel" | "Knob" => "◉",
        other => other,
    };
    n.to_string()
}
