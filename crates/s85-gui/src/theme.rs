//! Dark theme with an orange accent, plus the small building blocks every page is made of:
//! page headers, cards, option tiles and a segmented control.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontFamily, FontId, Frame, Margin, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, TextStyle, Vec2,
};

pub const ACCENT: Color32 = Color32::from_rgb(232, 106, 28);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(58, 36, 24);
pub const BG: Color32 = Color32::from_rgb(20, 20, 23);
pub const SIDEBAR: Color32 = Color32::from_rgb(15, 15, 17);
pub const SURFACE: Color32 = Color32::from_rgb(28, 28, 33);
pub const SURFACE_HI: Color32 = Color32::from_rgb(38, 38, 45);
pub const BORDER: Color32 = Color32::from_rgb(46, 46, 54);
pub const TEXT_MAIN: Color32 = Color32::from_rgb(232, 232, 238);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(150, 150, 164);
pub const OK: Color32 = Color32::from_rgb(70, 200, 110);
pub const DANGER: Color32 = Color32::from_rgb(235, 80, 80);

// Keyboard drawing.
pub const CASE: Color32 = Color32::from_rgb(30, 30, 36);
pub const KEY: Color32 = Color32::from_rgb(52, 52, 61);
pub const KEY_HOVER: Color32 = Color32::from_rgb(68, 68, 80);
pub const KEY_EDGE: Color32 = Color32::from_rgb(76, 76, 90);
/// Outline of keys that differ from the factory mapping.
pub const CHANGED: Color32 = Color32::from_rgb(80, 165, 255);
pub const TEXT: Color32 = Color32::from_rgb(226, 226, 234);
pub const DIM: Color32 = Color32::from_rgb(118, 118, 134);

pub fn apply(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = SURFACE;
    v.extreme_bg_color = Color32::from_rgb(14, 14, 16);
    v.faint_bg_color = SURFACE;
    v.override_text_color = Some(TEXT_MAIN);
    v.selection.bg_fill = ACCENT;
    v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
    v.hyperlink_color = ACCENT;
    v.slider_trailing_fill = true;
    v.window_corner_radius = CornerRadius::same(10);
    v.window_stroke = Stroke::new(1.0, BORDER);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active] {
        w.corner_radius = CornerRadius::same(6);
    }
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    v.widgets.inactive.weak_bg_fill = SURFACE_HI;
    v.widgets.inactive.bg_fill = SURFACE_HI;
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(50, 50, 59);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    v.widgets.active.weak_bg_fill = ACCENT;
    v.widgets.active.bg_fill = ACCENT;
    ctx.set_visuals(v);

    ctx.global_style_mut(|s| {
        s.spacing.item_spacing = Vec2::new(8.0, 8.0);
        s.spacing.button_padding = Vec2::new(10.0, 5.0);
        s.spacing.slider_width = 220.0;
        s.spacing.interact_size.y = 26.0;
        s.text_styles = [
            (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(22.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        ]
        .into();
    });
}

/// Icon + title + one-line explanation at the top of a page.
pub fn page_header(ui: &mut egui::Ui, icon: &str, title: &str, desc: &str) {
    ui.add_space(14.0);
    ui.horizontal(|ui| {
        icon_badge(ui, icon, 40.0);
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(RichText::new(title).heading().strong());
            ui.label(RichText::new(desc).color(TEXT_MUTED));
        });
    });
    ui.add_space(12.0);
}

/// An icon on a soft accent square.
pub fn icon_badge(ui: &mut egui::Ui, icon: &str, size: f32) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::same((size / 4.0) as u8), ACCENT_SOFT);
    ui.painter().text(r.center(), Align2::CENTER_CENTER, icon, FontId::proportional(size * 0.55), ACCENT);
}

pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    Frame::new()
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add(ui)
        })
        .inner
}

/// Title row of a card: icon, title and a muted description underneath.
pub fn card_header(ui: &mut egui::Ui, icon: &str, title: &str, desc: &str) {
    ui.horizontal(|ui| {
        icon_badge(ui, icon, 30.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(RichText::new(title).size(15.0).strong());
            ui.label(RichText::new(desc).size(12.5).color(TEXT_MUTED));
        });
    });
    ui.add_space(6.0);
}

/// Accent-filled button for the main action of a page.
pub fn primary_button(ui: &mut egui::Ui, enabled: bool, text: impl Into<String>) -> Response {
    let b = egui::Button::new(RichText::new(text.into()).size(15.0).strong().color(Color32::WHITE))
        .fill(ACCENT)
        .min_size(Vec2::new(0.0, 36.0))
        .corner_radius(CornerRadius::same(8));
    ui.add_enabled(enabled, b)
}

/// A small rounded label such as "24 frames".
pub fn chip(ui: &mut egui::Ui, icon: &str, text: impl Into<String>) {
    Frame::new().fill(SURFACE_HI).corner_radius(CornerRadius::same(255)).inner_margin(Margin::symmetric(10, 4)).show(
        ui,
        |ui| {
            ui.label(RichText::new(format!("{icon}  {}", text.into())).size(12.5).color(TEXT_MUTED));
        },
    );
}

/// One row of a sidebar: icon, title and a muted subtitle, highlighted when active.
pub fn nav_item(ui: &mut egui::Ui, icon: &str, title: &str, subtitle: &str, active: bool) -> Response {
    let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 50.0), Sense::click());
    let p = ui.painter();
    if active {
        p.rect_filled(r, CornerRadius::same(8), ACCENT_SOFT);
        p.rect_filled(Rect::from_min_size(r.min + Vec2::new(0.0, 12.0), Vec2::new(3.0, 26.0)), 2.0, ACCENT);
    } else if resp.hovered() {
        p.rect_filled(r, CornerRadius::same(8), SURFACE);
    }
    let fg = if active { ACCENT } else { TEXT_MAIN };
    p.text(r.left_center() + Vec2::new(24.0, 0.0), Align2::CENTER_CENTER, icon, FontId::proportional(20.0), fg);
    p.text(r.min + Vec2::new(44.0, 9.0), Align2::LEFT_TOP, title, FontId::proportional(14.5), fg);
    p.text(r.min + Vec2::new(44.0, 28.0), Align2::LEFT_TOP, subtitle, FontId::proportional(11.5), TEXT_MUTED);
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A selectable card with an icon, a title and a wrapped description.
pub fn option_tile(ui: &mut egui::Ui, icon: &str, title: &str, desc: &str, selected: bool, width: f32) -> Response {
    let text_w = width - 58.0;
    let title_g = ui.painter().layout(title.into(), FontId::proportional(14.0), TEXT_MAIN, text_w);
    let desc_g = ui.painter().layout(desc.into(), FontId::proportional(12.0), TEXT_MUTED, text_w);
    let h = (title_g.size().y + desc_g.size().y + 22.0).max(52.0);
    let (r, resp) = ui.allocate_exact_size(Vec2::new(width, h), Sense::click());
    let p = ui.painter();
    let (fill, stroke) = if selected {
        (ACCENT_SOFT, Stroke::new(1.5, ACCENT))
    } else if resp.hovered() {
        (SURFACE_HI, Stroke::new(1.0, BORDER))
    } else {
        (SURFACE, Stroke::new(1.0, BORDER))
    };
    p.rect_filled(r, CornerRadius::same(8), fill);
    p.rect_stroke(r, CornerRadius::same(8), stroke, StrokeKind::Inside);
    let fg = if selected { ACCENT } else { TEXT_MAIN };
    p.text(r.min + Vec2::new(24.0, h / 2.0), Align2::CENTER_CENTER, icon, FontId::proportional(20.0), fg);
    let ty = r.min.y + 10.0;
    let tx = r.min.x + 46.0;
    p.galley(egui::pos2(tx, ty), title_g.clone(), fg);
    p.galley(egui::pos2(tx, ty + title_g.size().y + 2.0), desc_g, TEXT_MUTED);
    if selected {
        p.text(
            r.right_top() + Vec2::new(-14.0, 14.0),
            Align2::CENTER_CENTER,
            crate::icons::CHECK,
            FontId::proportional(13.0),
            ACCENT,
        );
    }
    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// A row of connected buttons; returns the index clicked.
pub fn segmented(ui: &mut egui::Ui, items: &[(&str, &str)], selected: usize) -> Option<usize> {
    let seg_w = 118.0;
    let (r, _) = ui.allocate_exact_size(Vec2::new(seg_w * items.len() as f32 + 8.0, 38.0), Sense::hover());
    ui.painter().rect_filled(r, CornerRadius::same(9), SURFACE);
    ui.painter().rect_stroke(r, CornerRadius::same(9), Stroke::new(1.0, BORDER), StrokeKind::Inside);
    let mut clicked = None;
    for (i, (icon, label)) in items.iter().enumerate() {
        let sr = Rect::from_min_size(r.min + Vec2::new(4.0 + seg_w * i as f32, 4.0), Vec2::new(seg_w, 30.0));
        let resp = ui.interact(sr, ui.scope_id().with(("seg", i)), Sense::click());
        let active = i == selected;
        if active {
            ui.painter().rect_filled(sr, CornerRadius::same(6), ACCENT);
        } else if resp.hovered() {
            ui.painter().rect_filled(sr, CornerRadius::same(6), SURFACE_HI);
        }
        let fg = if active { Color32::WHITE } else { TEXT_MAIN };
        ui.painter().text(
            sr.center(),
            Align2::CENTER_CENTER,
            format!("{icon}  {label}"),
            FontId::proportional(14.0),
            fg,
        );
        if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
            clicked = Some(i);
        }
    }
    clicked
}

/// Muted explanatory text with an info icon.
pub fn hint(ui: &mut egui::Ui, text: &str) {
    ui.label(RichText::new(format!("{}  {text}", crate::icons::INFO)).size(12.5).color(TEXT_MUTED));
}
