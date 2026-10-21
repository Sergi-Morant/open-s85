//! Colour panel in the style of the official app: hue ring, R/G/B sliders and preset swatches.

use std::f32::consts::TAU;

use eframe::egui::{self, ecolor::Hsva, Color32, CornerRadius, Mesh, Pos2, RichText, Sense, Shape, Stroke, Vec2};

use crate::theme;

/// The official app's swatches, in its order.
pub const PRESETS: [[u8; 3]; 8] = [
    [255, 0, 0],
    [255, 128, 0],
    [255, 255, 0],
    [0, 255, 0],
    [0, 0, 255],
    [0, 255, 255],
    [255, 0, 255],
    [255, 255, 255],
];

const RING: f32 = 150.0;
const RING_WIDTH: f32 = 22.0;

/// Draws the panel for `rgb`; returns the new colour when the user changes it.
pub fn show(ui: &mut egui::Ui, rgb: [u8; 3]) -> Option<[u8; 3]> {
    let mut out = None;
    ui.horizontal_top(|ui| {
        if let Some(c) = hue_ring(ui, rgb) {
            out = Some(c);
        }
        ui.add_space(10.0);
        ui.vertical(|ui| {
            let mut v = rgb;
            for (i, (name, tint)) in
                [("R", Color32::from_rgb(235, 70, 70)), ("G", Color32::from_rgb(70, 200, 90)), ("B", Color32::from_rgb(80, 130, 255))]
                    .into_iter()
                    .enumerate()
            {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(name).strong().color(tint));
                    ui.scope(|ui| {
                        ui.visuals_mut().selection.bg_fill = tint;
                        ui.spacing_mut().slider_width = (ui.available_width() - 70.0).clamp(80.0, 200.0);
                        if ui.add(egui::Slider::new(&mut v[i], 0..=255).show_value(false)).changed() {
                            out = Some(v);
                        }
                    });
                    if ui.add(egui::DragValue::new(&mut v[i]).range(0..=255)).changed() {
                        out = Some(v);
                    }
                });
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new("#").color(theme::TEXT_MUTED));
                let id = ui.make_persistent_id("hex");
                let mut text = ui
                    .data(|d| d.get_temp::<String>(id))
                    .unwrap_or_else(|| format!("{:02X}{:02X}{:02X}", rgb[0], rgb[1], rgb[2]));
                let r = ui.add(egui::TextEdit::singleline(&mut text).desired_width(70.0).char_limit(6).font(egui::TextStyle::Monospace));
                if r.has_focus() {
                    if let Some(c) = parse_hex(&text) {
                        if r.changed() {
                            out = Some(c);
                        }
                    }
                    ui.data_mut(|d| d.insert_temp(id, text));
                } else {
                    // Not editing: always show the current colour.
                    ui.data_mut(|d| d.remove::<String>(id));
                }
            });
        });
    });
    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for p in PRESETS {
            let (r, resp) = ui.allocate_exact_size(Vec2::splat(26.0), Sense::click());
            ui.painter().rect_filled(r, CornerRadius::same(5), Color32::from_rgb(p[0], p[1], p[2]));
            if p == rgb || resp.hovered() {
                ui.painter().rect_stroke(r.expand(2.5), CornerRadius::same(7), Stroke::new(2.0, Color32::WHITE), egui::StrokeKind::Inside);
            }
            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                out = Some(p);
            }
        }
    });
    out.filter(|&c| c != rgb)
}

fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(s, 16).ok()?;
    Some([(n >> 16) as u8, (n >> 8) as u8, n as u8])
}

fn hue_color(h: f32) -> Color32 {
    Hsva::new(h.rem_euclid(1.0), 1.0, 1.0, 1.0).into()
}

/// Point on a circle for hue `h`: red at the right, going counter-clockwise like the official app.
fn at(center: Pos2, radius: f32, h: f32) -> Pos2 {
    center + Vec2::new((h * TAU).cos(), -(h * TAU).sin()) * radius
}

/// A ring of fully saturated hues with the current colour in the middle; click or drag to pick.
fn hue_ring(ui: &mut egui::Ui, rgb: [u8; 3]) -> Option<[u8; 3]> {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(RING), Sense::click_and_drag());
    let c = rect.center();
    let outer = RING / 2.0 - 2.0;
    let inner = outer - RING_WIDTH;

    let mut mesh = Mesh::default();
    const SEGMENTS: u32 = 120;
    for i in 0..=SEGMENTS {
        let h = i as f32 / SEGMENTS as f32;
        let col = hue_color(h);
        mesh.colored_vertex(at(c, outer, h), col);
        mesh.colored_vertex(at(c, inner, h), col);
        if i > 0 {
            let b = i * 2;
            mesh.add_triangle(b - 2, b - 1, b);
            mesh.add_triangle(b - 1, b + 1, b);
        }
    }
    let p = ui.painter();
    p.add(Shape::mesh(mesh));

    let current = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    p.circle_filled(c, inner - 10.0, current);
    p.circle_stroke(c, inner - 10.0, Stroke::new(1.0, theme::BORDER));

    // Marker at the current hue (only meaningful for coloured, not grey, values).
    let hsva = Hsva::from(current);
    if hsva.s > 0.05 {
        let m = at(c, (outer + inner) / 2.0, hsva.h);
        p.circle_filled(m, RING_WIDTH / 2.0 - 1.0, hue_color(hsva.h));
        p.circle_stroke(m, RING_WIDTH / 2.0 - 1.0, Stroke::new(2.5, Color32::WHITE));
    }

    let pos = resp.interact_pointer_pos()?;
    let d = pos - c;
    // Start a pick only on the ring itself; once dragging, follow the pointer anywhere.
    if !resp.dragged() && !(inner - 4.0..=outer + 4.0).contains(&d.length()) {
        return None;
    }
    ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
    let h = (-d.y).atan2(d.x) / TAU;
    let col = hue_color(h);
    Some([col.r(), col.g(), col.b()])
}
