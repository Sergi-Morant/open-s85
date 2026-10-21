#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod color_panel;
mod icons;
mod keyboard_view;
mod profile;
mod theme;
mod worker;

use std::path::PathBuf;
use std::time::Duration;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Frame, Layout, Margin, RichText, Sense, Stroke, Vec2,
};
use s85_protocol::keycode::{catalog, Category, KeyCode};
use s85_protocol::keymap::Layer;
use s85_protocol::layout::{key_at_slot, KEYS};
use s85_protocol::lighting::{self, Config};
use s85_protocol::screen::{self, Animation, Fit};

use keyboard_view::KeyLook;
use profile::{Profile, Store};
use theme::{card, card_header, hint, page_header};
use worker::{Event, Job, Worker};

const IMAGE_EXTS: [&str; 6] = ["gif", "png", "jpg", "jpeg", "bmp", "webp"];

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Open S85")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([980.0, 640.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    eframe::run_native("Open S85", options, Box::new(|cc| Ok(Box::new(App::new(cc)))))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Keys,
    Lighting,
    Screen,
    Settings,
}

impl Page {
    const ALL: [(Page, &'static str, &'static str, &'static str); 4] = [
        (Page::Keys, icons::KEYBOARD, "Keys", "Remap keys and layers"),
        (Page::Lighting, icons::LIGHTBULB, "Lighting", "Backlight effects"),
        (Page::Screen, icons::MONITOR, "Screen", "Images, GIFs and clock"),
        (Page::Settings, icons::GEAR, "Settings", "Profiles and about"),
    ];
}

fn layer_info(l: Layer) -> (&'static str, &'static str) {
    match l {
        Layer::Default => (icons::KEYBOARD, "What every key types normally."),
        Layer::Fn1 => {
            (icons::ARROW_FAT_UP, "Active while you hold the Fn key. Empty keys show their normal legend dimmed.")
        }
        Layer::Fn2 => (
            icons::ARROW_FAT_LINES_UP,
            "A second Fn layer. Only written to the keyboard once you assign something here.",
        ),
        Layer::Tap => (icons::HAND_TAP, "RK's Tap layer. Only written to the keyboard once you assign something here."),
    }
}

fn category_icon(c: Category) -> &'static str {
    match c {
        Category::Number => icons::HASH,
        Category::Characters => icons::TEXT_AA,
        Category::Modifier => icons::COMMAND,
        Category::Function => icons::FUNCTION,
        Category::Arrow => icons::ARROWS_DOWN_UP,
        Category::Numpad => icons::CALCULATOR,
        Category::Media => icons::MUSIC_NOTES,
        Category::Other => icons::DOTS_THREE,
    }
}

/// Icon and one-line description for each lighting effect.
fn mode_info(m: u8) -> (&'static str, &'static str) {
    match m {
        lighting::MODE_OFF => (icons::POWER, "Backlight switched off."),
        1 => (icons::CIRCLE, "One steady colour on every key."),
        2 => (icons::PULSE, "Keys slowly fade in and out, like breathing."),
        3 => (icons::RAINBOW, "A rainbow sweeps across the keyboard."),
        4 => (icons::LIGHTNING, "Keys flash when pressed, then fade away."),
        5 => (icons::DROP, "Random keys light up like falling rain."),
        6 => (icons::SPIRAL, "Rainbow colours turn around the board."),
        7 => (icons::RADIO_BUTTON, "Ripples of light spread from each key press."),
        8 => (icons::STAR, "Random keys twinkle like stars."),
        9 => (icons::CIRCLE_HALF, "Pressed keys go dark, then light up again."),
        10 => (icons::WAVE_TRIANGLE, "A trail of light snakes along the rows."),
        11 => (icons::BROADCAST, "Neon colours stream across the keys."),
        12 => (icons::HAND_TAP, "Keys light up as you type."),
        13 => (icons::WAVE_SINE, "A smooth wave of light rolls across the board."),
        14 => (icons::SCAN, "A bar of light scans back and forth."),
        15 => (icons::FAN, "Colours spin like windmill blades."),
        16 => (icons::WAVES, "Colours cascade down the keyboard."),
        17 => (icons::FLOWER, "Light blossoms outward across the keys."),
        _ => (icons::LIGHTBULB, ""),
    }
}

#[derive(Default)]
struct ScreenState {
    path: Option<PathBuf>,
    fit: Fit,
    delay: Option<u16>,
    anim: Option<Animation>,
    frames: Vec<egui::TextureHandle>,
    paused: bool,
    progress: Option<(usize, usize)>,
}

struct App {
    worker: Worker,
    store: Store,
    page: Page,
    device: Option<String>,
    status: String,
    status_is_error: bool,
    cfg: Option<Config>,
    layer: Layer,
    selected: Option<u8>,
    screen: ScreenState,
    renaming: Option<String>,
    search: String,
    confirm_reset: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> App {
        icons::install(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);
        let worker = Worker::spawn(cc.egui_ctx.clone());
        worker.send(Job::Connect);
        App {
            worker,
            store: Store::load(),
            page: Page::Keys,
            device: None,
            status: "Connecting...".into(),
            status_is_error: false,
            cfg: None,
            layer: Layer::Default,
            selected: None,
            screen: ScreenState::default(),
            renaming: None,
            search: String::new(),
            confirm_reset: false,
        }
    }

    fn info(&mut self, s: impl Into<String>) {
        self.status = s.into();
        self.status_is_error = false;
    }

    fn error(&mut self, s: impl Into<String>) {
        self.status = s.into();
        self.status_is_error = true;
    }

    fn save(&mut self) {
        if let Err(e) = self.store.save() {
            self.error(format!("Could not save profiles: {e}"));
        }
    }

    fn poll(&mut self) {
        while let Ok(ev) = self.worker.rx.try_recv() {
            match ev {
                Event::Connected(name) => {
                    self.info(format!("Connected to {name}"));
                    self.device = Some(name);
                }
                Event::Disconnected(e) => {
                    self.device = None;
                    self.error(e);
                }
                Event::Config(c) => self.cfg = Some(c),
                Event::Progress(done, total) => self.screen.progress = Some((done, total)),
                Event::Done(msg) => {
                    self.screen.progress = None;
                    self.info(msg);
                }
                Event::Error(e) => {
                    self.screen.progress = None;
                    self.device = None;
                    self.error(format!("Keyboard error: {e}"));
                }
            }
        }
    }

    fn write_config(&mut self) {
        if let Some(c) = &self.cfg {
            self.worker.send(Job::WriteConfig(c.clone()));
        }
    }

    fn write_profile(&mut self) {
        let p = self.store.current().clone();
        self.worker.send(Job::WriteAll(Box::new(p.keymap()), p.palettes()));
    }
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();

        egui::Panel::left("nav")
            .exact_size(220.0)
            .resizable(false)
            .frame(Frame::new().fill(theme::SIDEBAR).inner_margin(Margin::same(12)))
            .show(root, |ui| self.sidebar(ui));

        egui::Panel::bottom("status")
            .frame(Frame::new().fill(theme::SIDEBAR).inner_margin(Margin::symmetric(16, 6)))
            .show(root, |ui| {
                ui.horizontal(|ui| {
                    let (icon, c) = if self.status_is_error {
                        (icons::WARNING, theme::DANGER)
                    } else {
                        (icons::INFO, theme::TEXT_MUTED)
                    };
                    ui.label(RichText::new(format!("{icon}  {}", self.status)).size(12.5).color(c));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let name = self.store.current().name.clone();
                        ui.label(RichText::new(format!("{}  {name}", icons::USER)).size(12.5).color(theme::TEXT_MUTED));
                    });
                });
            });

        match self.page {
            Page::Keys => self.keys_page(root),
            Page::Lighting => self.lighting_page(root),
            Page::Screen => self.screen_page(root),
            Page::Settings => self.settings_page(root),
        }
    }
}

// ---------------------------------------------------------------- Sidebar

impl App {
    fn sidebar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("connection").frame(Frame::NONE).show(ui, |ui| self.connection_card(ui));
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            theme::icon_badge(ui, icons::KEYBOARD, 36.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new("Open S85").size(17.0).strong());
                ui.label(RichText::new(format!("v{}", env!("CARGO_PKG_VERSION"))).size(11.5).color(theme::TEXT_MUTED));
            });
        });
        ui.add_space(18.0);
        ui.label(RichText::new("MENU").size(11.0).color(theme::TEXT_MUTED));
        ui.add_space(2.0);
        for (page, icon, title, subtitle) in Page::ALL {
            if theme::nav_item(ui, icon, title, subtitle, self.page == page).clicked() {
                self.page = page;
            }
            ui.add_space(2.0);
        }
    }

    fn connection_card(&mut self, ui: &mut egui::Ui) {
        Frame::new()
            .fill(theme::SURFACE)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                let (icon, color, title) = match &self.device {
                    Some(_) => (icons::PLUGS_CONNECTED, theme::OK, "Connected"),
                    None => (icons::PLUGS, theme::DANGER, "Not connected"),
                };
                ui.horizontal(|ui| {
                    ui.label(RichText::new(icon).size(20.0).color(color));
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new(title).strong().color(color));
                        let sub = self.device.clone().unwrap_or_else(|| "Plug in the keyboard via USB".into());
                        ui.label(RichText::new(sub).size(11.5).color(theme::TEXT_MUTED));
                    });
                });
                if self.device.is_none() {
                    ui.add_space(4.0);
                    if ui.button(format!("{}  Reconnect", icons::REFRESH)).clicked() {
                        self.worker.send(Job::Connect);
                    }
                }
            });
    }
}

// ---------------------------------------------------------------- Keys

impl App {
    fn profiles_panel(&mut self, root: &mut egui::Ui) {
        egui::Panel::left("profiles")
            .exact_size(240.0)
            .resizable(false)
            .frame(Frame::new().fill(theme::BG).inner_margin(Margin::same(14)).stroke(Stroke::new(1.0, theme::BORDER)))
            .show(root, |ui| {
                ui.add_space(4.0);
                card_header(
                    ui,
                    icons::USERS,
                    "Profiles",
                    "Saved on this computer. Selecting one writes it to the keyboard.",
                );
                ui.horizontal(|ui| {
                    if ui
                        .button(format!("{}  New", icons::PLUS))
                        .on_hover_text("Create a profile with the factory mapping")
                        .clicked()
                    {
                        let n = self.store.profiles.len() + 1;
                        self.store.profiles.push(Profile::new(&format!("Profile{n}")));
                        self.save();
                    }
                    if ui.button(icons::PENCIL).on_hover_text("Rename this profile").clicked() {
                        self.renaming = Some(self.store.current().name.clone());
                    }
                    let can_delete = self.store.profiles.len() > 1;
                    if ui
                        .add_enabled(can_delete, egui::Button::new(icons::TRASH))
                        .on_hover_text("Delete this profile")
                        .on_disabled_hover_text("At least one profile must remain")
                        .clicked()
                    {
                        self.store.profiles.remove(self.store.active);
                        self.store.active = self.store.active.saturating_sub(1);
                        self.save();
                    }
                });
                if let Some(name) = &mut self.renaming {
                    let r = ui.add(egui::TextEdit::singleline(name).hint_text("Profile name"));
                    if r.lost_focus() {
                        let name = std::mem::take(name);
                        if !name.trim().is_empty() {
                            self.store.current().name = name.trim().to_string();
                            self.save();
                        }
                        self.renaming = None;
                    } else {
                        r.request_focus();
                    }
                }
                ui.add_space(6.0);
                let mut switch_to = None;
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (i, p) in self.store.profiles.iter().enumerate() {
                        let active = i == self.store.active;
                        let sub = if active { "Active" } else { "Click to load" };
                        if theme::nav_item(ui, icons::USER, &p.name, sub, active).clicked() && !active {
                            switch_to = Some(i);
                        }
                    }
                });
                if let Some(i) = switch_to {
                    self.store.active = i;
                    self.save();
                    self.write_profile();
                }
            });
    }

    fn keys_page(&mut self, root: &mut egui::Ui) {
        self.profiles_panel(root);
        egui::CentralPanel::default().frame(Frame::new().fill(theme::BG).inner_margin(Margin::symmetric(22, 0))).show(
            root,
            |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    page_header(
                        ui,
                        icons::KEYBOARD,
                        "Key mapping",
                        "Give any key a new function on each layer. Changes are sent to the keyboard immediately.",
                    );
                    let items: Vec<(&str, &str)> = Layer::ALL.iter().map(|&l| (layer_info(l).0, l.title())).collect();
                    let idx = Layer::ALL.iter().position(|&l| l == self.layer).unwrap_or(0);
                    if let Some(i) = theme::segmented(ui, &items, idx) {
                        self.layer = Layer::ALL[i];
                    }
                    hint(ui, layer_info(self.layer).1);
                    ui.add_space(8.0);

                    let km = self.store.current().keymap();
                    let factory = s85_protocol::keymap::Keymap::default();
                    let layer = self.layer;
                    let sel: Vec<u8> = self.selected.into_iter().collect();
                    let clicked = keyboard_view::show(ui, &sel, |k| {
                        let code = km.get(layer, k.slot);
                        let empty = code == KeyCode::NONE && layer != Layer::Default;
                        KeyLook {
                            text: if empty {
                                keyboard_view::short(k.label)
                            } else {
                                keyboard_view::short(&code.name())
                            },
                            dim: empty,
                            changed: code != factory.get(layer, k.slot),
                            fill: None,
                        }
                    });
                    if let Some(s) = clicked {
                        self.selected = if self.selected == Some(s) { None } else { Some(s) };
                    }
                    ui.horizontal(|ui| {
                        legend(ui, theme::ACCENT, "Selected");
                        legend(ui, theme::CHANGED, "Customised");
                        legend(ui, theme::DIM, "Empty on this layer");
                    });
                    ui.add_space(8.0);

                    card(ui, |ui| match self.selected.and_then(key_at_slot) {
                        Some(k) => {
                            let slot = k.slot;
                            ui.horizontal(|ui| {
                                card_header(
                                    ui,
                                    icons::CURSOR_CLICK,
                                    &format!("{} on {}", k.label, layer.title()),
                                    &format!("Currently: {}. Pick a new function below.", km.get(layer, slot).name()),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button(icons::X).on_hover_text("Deselect").clicked() {
                                        self.selected = None;
                                    }
                                    if ui
                                        .button(format!("{}  Factory", icons::RESTORE))
                                        .on_hover_text("Restore this key's factory function")
                                        .clicked()
                                    {
                                        self.assign(slot, factory.get(layer, slot));
                                    }
                                });
                            });
                        }
                        None => card_header(
                            ui,
                            icons::CURSOR_CLICK,
                            "No key selected",
                            "Click a key on the keyboard above, then choose its new function below.",
                        ),
                    });
                    ui.add_space(10.0);
                    self.key_picker(ui);
                    ui.add_space(16.0);
                });
            },
        );
    }

    fn key_picker(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(icons::MAGNIFIER).size(16.0).color(theme::TEXT_MUTED));
            ui.add(
                egui::TextEdit::singleline(&mut self.search)
                    .hint_text("Search functions, e.g. \"volume\" or \"F5\"")
                    .desired_width(320.0),
            );
            if !self.search.is_empty() && ui.button(icons::X).on_hover_text("Clear search").clicked() {
                self.search.clear();
            }
        });
        ui.add_space(6.0);
        let needle = self.search.to_lowercase();
        let cats: Vec<Category> = Category::ALL
            .into_iter()
            .filter(|&c| catalog().iter().any(|e| e.category == c && e.name.to_lowercase().contains(&needle)))
            .collect();
        if cats.is_empty() {
            hint(ui, "No function matches that search.");
            return;
        }
        let mut pick = None;
        let enabled = self.selected.is_some();
        let ncols = ((ui.available_width() / 300.0).floor() as usize).clamp(1, 3);
        for row in cats.chunks(ncols) {
            ui.columns(ncols, |cols| {
                for (col, &cat) in cols.iter_mut().zip(row) {
                    card(col, |ui| {
                        ui.label(
                            RichText::new(format!("{}  {}", category_icon(cat), cat.title()))
                                .strong()
                                .size(14.5)
                                .color(theme::ACCENT),
                        );
                        ui.add_space(2.0);
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(5.0, 5.0);
                            for e in catalog()
                                .iter()
                                .filter(|e| e.category == cat && e.name.to_lowercase().contains(&needle))
                            {
                                let r = ui
                                    .add_enabled(enabled, egui::Button::new(e.name))
                                    .on_disabled_hover_text("Select a key on the keyboard first");
                                if r.clicked() {
                                    pick = Some(e.code);
                                }
                            }
                        });
                    });
                }
            });
            ui.add_space(8.0);
        }
        if let (Some(code), Some(slot)) = (pick, self.selected) {
            self.assign(slot, code);
        }
    }

    fn assign(&mut self, slot: u8, code: KeyCode) {
        let layer = self.layer;
        let mut km = self.store.current().keymap();
        km.set(layer, slot, code);
        if !km.has_fn_key() {
            self.error("Refused: the Default layer must keep at least one Fn key.");
            return;
        }
        self.store.current().set_keymap(&km);
        self.save();
        self.worker.send(Job::WriteLayer(Box::new(km), layer));
        self.info(format!("{} set to {}", key_at_slot(slot).map_or("?", |k| k.label), code.name()));
    }
}

fn legend(ui: &mut egui::Ui, color: Color32, text: &str) {
    let (r, _) = ui.allocate_exact_size(Vec2::splat(12.0), Sense::hover());
    ui.painter().rect_stroke(r, CornerRadius::same(3), Stroke::new(2.0, color), egui::StrokeKind::Inside);
    ui.label(RichText::new(text).size(12.0).color(theme::TEXT_MUTED));
    ui.add_space(8.0);
}

fn not_connected(ui: &mut egui::Ui, worker: &Worker, what: &str) {
    ui.add_space(60.0);
    ui.vertical_centered(|ui| {
        ui.set_max_width(420.0);
        card(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(icons::PLUGS).size(42.0).color(theme::TEXT_MUTED));
                ui.label(RichText::new("Keyboard not connected").size(17.0).strong());
                ui.label(RichText::new(what).color(theme::TEXT_MUTED));
                ui.add_space(6.0);
                if theme::primary_button(ui, true, format!("{}  Connect", icons::REFRESH)).clicked() {
                    worker.send(Job::Connect);
                }
            });
        });
    });
}

// ---------------------------------------------------------------- Lighting

impl App {
    fn lighting_page(&mut self, root: &mut egui::Ui) {
        let Some(mut cfg) = self.cfg.clone() else {
            egui::CentralPanel::default().show(root, |ui| {
                not_connected(
                    ui,
                    &self.worker,
                    "Lighting settings are read from the keyboard, so it needs to be plugged in.",
                );
            });
            return;
        };
        let before = cfg.clone();
        let mut palette_changed = false;

        egui::Panel::left("modes")
            .exact_size(280.0)
            .resizable(false)
            .frame(Frame::new().fill(theme::BG).inner_margin(Margin::same(14)).stroke(Stroke::new(1.0, theme::BORDER)))
            .show(root, |ui| {
                ui.add_space(4.0);
                card_header(ui, icons::SWATCHES, "Effects", "How the backlight animates.");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    // Leave room for the floating scrollbar.
                    let w = ui.available_width() - 12.0;
                    for &(m, name) in lighting::MODES.iter().chain([(lighting::MODE_OFF, "Off")].iter()) {
                        let (icon, desc) = mode_info(m);
                        if theme::option_tile(ui, icon, name, desc, cfg.mode() == m, w).clicked() {
                            cfg.set_mode(m);
                        }
                        ui.add_space(2.0);
                    }
                });
            });

        egui::CentralPanel::default().frame(Frame::new().fill(theme::BG).inner_margin(Margin::symmetric(22, 0))).show(
            root,
            |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let m = cfg.mode();
                    let name = lighting::MODES.iter().find(|e| e.0 == m).map_or("Off", |e| e.1);
                    let (icon, desc) = mode_info(m);
                    page_header(ui, icon, &format!("Lighting · {name}"), desc);
                    let mut pal = self.store.current().palettes();
                    let color = cfg.color(m);
                    let rgb = pal.0[m as usize][(color as usize).min(6)];
                    keyboard_view::show(ui, &[], |k| KeyLook {
                        text: keyboard_view::short(k.label),
                        dim: false,
                        changed: false,
                        fill: (m != lighting::MODE_OFF && color != lighting::COLOR_MIX && lighting::mode_has_color(m))
                            .then(|| Color32::from_rgb(rgb[0], rgb[1], rgb[2]).gamma_multiply(0.85)),
                    });
                    hint(
                        ui,
                        "Preview shows the chosen colour; animations only play on the keyboard. Changes apply live.",
                    );
                    ui.add_space(10.0);
                    if m == lighting::MODE_OFF {
                        card(ui, |ui| {
                            card_header(
                                ui,
                                icons::POWER,
                                "Backlight is off",
                                "Pick an effect on the left to turn it back on.",
                            );
                        });
                        sleep_card(ui, &mut cfg);
                        return;
                    }
                    ui.columns(2, |cols| {
                        let ui = &mut cols[0];
                        card(ui, |ui| {
                            card_header(ui, icons::SUN, "Brightness", "How bright the backlight is for this effect.");
                            let mut b = cfg.brightness(m);
                            let r = ui.add(
                                egui::Slider::new(&mut b, 0..=lighting::MAX_BRIGHTNESS)
                                    .step_by(lighting::BRIGHTNESS_STEP as f64)
                                    .show_value(false),
                            );
                            if r.changed() {
                                cfg.set_brightness(m, b);
                            }
                        });
                        if lighting::mode_has_speed(m) {
                            ui.add_space(8.0);
                            card(ui, |ui| {
                                card_header(ui, icons::GAUGE, "Speed", "How fast the effect moves.");
                                let mut s = cfg.speed(m);
                                if ui
                                    .add(egui::Slider::new(&mut s, 0..=lighting::MAX_SPEED).show_value(false))
                                    .changed()
                                {
                                    cfg.set_speed(m, s);
                                }
                            });
                        }
                        ui.add_space(8.0);
                        sleep_card(ui, &mut cfg);

                        let ui = &mut cols[1];
                        card(ui, |ui| {
                            card_header(
                                ui,
                                icons::PALETTE,
                                "Colour",
                                "One fixed colour, or let the effect mix colours itself.",
                            );
                            if !lighting::mode_has_color(m) {
                                hint(ui, "This effect uses its own built-in colours.");
                                return;
                            }
                            let mut mix = color == lighting::COLOR_MIX;
                            if ui.checkbox(&mut mix, "Colour mixing (rainbow)").changed() {
                                cfg.set_color(m, if mix { lighting::COLOR_MIX } else { 0 });
                            }
                            ui.add_space(4.0);
                            let new_rgb = ui.add_enabled_ui(!mix, |ui| color_panel::show(ui, rgb)).inner;
                            if let Some(v) = new_rgb {
                                // Like the official app: the chosen colour goes into palette slot 0 and is selected.
                                pal.0[m as usize][0] = v;
                                cfg.set_color(m, 0);
                                self.store.current().set_palettes(&pal);
                                palette_changed = true;
                            }
                        });
                    });
                    ui.add_space(16.0);
                });
            },
        );

        if palette_changed {
            self.save();
            self.worker.send(Job::WritePalettes(self.store.current().palettes()));
        }
        if cfg != before {
            self.cfg = Some(cfg);
            // Sliders send on every step; writes are tiny (one 520-byte report), so this keeps it live.
            self.write_config();
        }
    }
}

fn sleep_card(ui: &mut egui::Ui, cfg: &mut Config) {
    card(ui, |ui| {
        card_header(
            ui,
            icons::MOON,
            "Wireless sleep",
            "On Bluetooth or 2.4 GHz, sleep after this long without typing.",
        );
        let cur = cfg.sleep_minutes();
        let label = |v: u8| {
            if v == 0 {
                "Never".to_string()
            } else {
                format!("{v} min")
            }
        };
        egui::ComboBox::from_id_salt("sleep").selected_text(label(cur)).show_ui(ui, |ui| {
            for v in lighting::SLEEP_MINUTES {
                if ui.selectable_label(cur == v, label(v)).clicked() {
                    cfg.set_sleep_minutes(v);
                }
            }
        });
    });
}

// ---------------------------------------------------------------- Screen

impl App {
    fn load_image(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screen.path.clone() else {
            return;
        };
        match Animation::load(&path, self.screen.fit, self.screen.delay) {
            Ok(anim) => {
                self.screen.frames = anim
                    .frames
                    .iter()
                    .enumerate()
                    .map(|(i, f)| {
                        let img = egui::ColorImage::from_rgb([f.width() as usize, f.height() as usize], f.as_raw());
                        ctx.load_texture(format!("frame{i}"), img, Default::default())
                    })
                    .collect();
                self.screen.anim = Some(anim);
            }
            Err(e) => self.error(format!("Could not load image: {e}")),
        }
    }

    fn pick_image(&mut self) -> bool {
        match rfd::FileDialog::new().add_filter("Images", &IMAGE_EXTS).pick_file() {
            Some(p) => {
                self.screen.path = Some(p);
                true
            }
            None => false,
        }
    }

    fn screen_page(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        let mut reload = false;

        let (dropped, dragging) = ctx
            .input(|i| (i.raw.dropped_files.first().map(|f| f.path().to_path_buf()), !i.raw.hovered_files.is_empty()));
        if let Some(p) = dropped {
            let ok = p.extension().is_some_and(|e| IMAGE_EXTS.iter().any(|x| e.eq_ignore_ascii_case(x)));
            if ok {
                self.screen.path = Some(p);
                reload = true;
            } else {
                self.error("That file type is not supported. Use GIF, PNG, JPG, BMP or WebP.");
            }
        }

        egui::CentralPanel::default().frame(Frame::new().fill(theme::BG).inner_margin(Margin::symmetric(22, 0))).show(
            root,
            |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    page_header(
                        ui,
                        icons::MONITOR,
                        "Screen",
                        "Put a picture or an animated GIF on the keyboard's 320 × 172 display.",
                    );
                    ui.columns(2, |cols| {
                        if self.preview_card(&mut cols[0], dragging) {
                            reload |= self.pick_image();
                        }
                        let ui = &mut cols[1];
                        card(ui, |ui| {
                            card_header(
                                ui,
                                icons::FILE_IMAGE,
                                "Image",
                                "GIF, PNG, JPG, BMP or WebP. Animations are capped at 60 frames.",
                            );
                            ui.horizontal(|ui| {
                                let name = self
                                    .screen
                                    .path
                                    .as_ref()
                                    .and_then(|p| p.file_name())
                                    .map(|n| n.to_string_lossy().to_string());
                                match name {
                                    Some(n) => ui.label(RichText::new(n).strong()),
                                    None => ui.label(RichText::new("No file chosen").color(theme::TEXT_MUTED)),
                                };
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button(format!("{}  Browse…", icons::FILE_IMAGE)).clicked() {
                                        reload |= self.pick_image();
                                    }
                                });
                            });
                        });
                        ui.add_space(8.0);
                        card(ui, |ui| {
                            card_header(ui, icons::FRAME_CORNERS, "Fit", "How the image is scaled to the screen.");
                            let w = ui.available_width();
                            for (f, icon, name, desc) in [
                                (
                                    Fit::Cover,
                                    icons::CROP,
                                    "Fill",
                                    "Covers the whole screen; edges that don't fit are cropped.",
                                ),
                                (
                                    Fit::Contain,
                                    icons::FRAME_CORNERS,
                                    "Fit",
                                    "Shows the whole image, with black bars if needed.",
                                ),
                                (
                                    Fit::Stretch,
                                    icons::ARROWS_OUT,
                                    "Stretch",
                                    "Fills the screen exactly, distorting proportions.",
                                ),
                            ] {
                                if theme::option_tile(ui, icon, name, desc, self.screen.fit == f, w).clicked()
                                    && self.screen.fit != f
                                {
                                    self.screen.fit = f;
                                    reload = true;
                                }
                                ui.add_space(2.0);
                            }
                        });
                        ui.add_space(8.0);
                        card(ui, |ui| {
                            card_header(
                                ui,
                                icons::TIMER,
                                "Frame timing",
                                "GIFs keep their own speed unless you override it here.",
                            );
                            let mut custom = self.screen.delay.is_some();
                            if ui.checkbox(&mut custom, "Custom delay between frames").changed() {
                                self.screen.delay = custom.then_some(100);
                                reload = true;
                            }
                            if let Some(d) = &mut self.screen.delay {
                                let r = ui.add(egui::Slider::new(d, 10..=1000).logarithmic(true).suffix(" ms"));
                                if r.drag_stopped() || (r.changed() && !r.dragged()) {
                                    reload = true;
                                }
                            }
                        });
                        ui.add_space(8.0);
                        self.upload_card(ui, &ctx);
                        ui.add_space(8.0);
                        card(ui, |ui| {
                            card_header(
                                ui,
                                icons::CLOCK,
                                "Clock",
                                "The screen shows the time; it is synced automatically on connect.",
                            );
                            if ui
                                .add_enabled(
                                    self.device.is_some(),
                                    egui::Button::new(format!("{}  Sync now", icons::REFRESH)),
                                )
                                .clicked()
                            {
                                self.worker.send(Job::SyncClock);
                            }
                        });
                    });
                    ui.add_space(16.0);
                });
            },
        );

        if reload {
            self.load_image(&ctx);
        }
    }

    /// The simulated TFT. Returns true when the empty drop zone was clicked.
    fn preview_card(&mut self, ui: &mut egui::Ui, dragging: bool) -> bool {
        card(ui, |ui| {
            card_header(ui, icons::FILM_STRIP, "Preview", "How it will look on the keyboard's display.");
            let w = ui.available_width();
            let bezel = 12.0;
            let screen_w = w - bezel * 2.0;
            let screen_h = screen_w * screen::HEIGHT as f32 / screen::WIDTH as f32;
            let (outer, resp) = ui.allocate_exact_size(Vec2::new(w, screen_h + bezel * 2.0), Sense::click());
            let inner = outer.shrink(bezel);
            let p = ui.painter();
            p.rect_filled(outer, CornerRadius::same(12), Color32::from_rgb(10, 10, 12));
            p.rect_stroke(outer, CornerRadius::same(12), Stroke::new(1.0, theme::BORDER), egui::StrokeKind::Inside);

            let mut clicked = false;
            if self.screen.frames.is_empty() {
                let hot = dragging || resp.hovered();
                let c = if hot { theme::ACCENT } else { theme::TEXT_MUTED };
                p.rect_filled(inner, CornerRadius::same(6), if hot { theme::ACCENT_SOFT } else { theme::SURFACE });
                dashed_rect(p, inner, Stroke::new(1.5, c));
                p.text(
                    inner.center() - Vec2::new(0.0, 20.0),
                    Align2::CENTER_CENTER,
                    icons::FILE_IMAGE,
                    FontId::proportional(38.0),
                    c,
                );
                let msg = if dragging { "Drop to load" } else { "Drop an image or GIF here" };
                p.text(
                    inner.center() + Vec2::new(0.0, 16.0),
                    Align2::CENTER_CENTER,
                    msg,
                    FontId::proportional(15.0),
                    theme::TEXT_MAIN,
                );
                p.text(
                    inner.center() + Vec2::new(0.0, 36.0),
                    Align2::CENTER_CENTER,
                    "or click to browse",
                    FontId::proportional(12.5),
                    theme::TEXT_MUTED,
                );
                clicked = resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked();
            } else {
                let n = self.screen.frames.len();
                let delay = self.screen.anim.as_ref().map_or(100, |a| a.delay_ms).max(10) as u64;
                let idx = if self.screen.paused || n == 1 {
                    0
                } else {
                    ui.ctx().request_repaint_after(Duration::from_millis(delay));
                    ((ui.input(|i| i.time) * 1000.0) as u64 / delay) as usize % n
                };
                let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                p.image(self.screen.frames[idx].id(), inner, uv, Color32::WHITE);
                if dragging {
                    p.rect_filled(inner, CornerRadius::same(4), Color32::from_black_alpha(170));
                    p.text(
                        inner.center(),
                        Align2::CENTER_CENTER,
                        format!("{}  Drop to replace", icons::FILE_IMAGE),
                        FontId::proportional(16.0),
                        theme::ACCENT,
                    );
                }
            }
            ui.add_space(6.0);

            if let Some(anim) = &self.screen.anim {
                ui.horizontal_wrapped(|ui| {
                    let n = anim.frames.len();
                    if n > 1 {
                        let (icon, tip) = if self.screen.paused {
                            (icons::PLAY, "Play preview")
                        } else {
                            (icons::PAUSE, "Pause preview")
                        };
                        if ui.button(icon).on_hover_text(tip).clicked() {
                            self.screen.paused = !self.screen.paused;
                        }
                    }
                    theme::chip(
                        ui,
                        icons::FILM_STRIP,
                        if n == 1 { "Still image".to_string() } else { format!("{n} frames") },
                    );
                    if n > 1 {
                        theme::chip(ui, icons::TIMER, format!("{} ms / frame", anim.delay_ms));
                    }
                    let secs = n as f32 * screen::CHUNKS_PER_FRAME as f32 * 0.073;
                    theme::chip(ui, icons::HOURGLASS, format!("~{:.0} s upload", secs.max(1.0)));
                });
            }
            clicked
        })
    }

    fn upload_card(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        card(ui, |ui| {
            card_header(
                ui,
                icons::UPLOAD,
                "Send to keyboard",
                "Replaces what the screen shows now. Keep the keyboard plugged in.",
            );
            match self.screen.progress {
                Some((done, total)) => {
                    ui.add(
                        egui::ProgressBar::new(done as f32 / total.max(1) as f32)
                            .show_percentage()
                            .fill(theme::ACCENT)
                            .desired_height(22.0),
                    );
                    hint(ui, "Uploading… you can keep using the app.");
                }
                None => {
                    let ready = self.device.is_some() && self.screen.anim.is_some();
                    let r = theme::primary_button(ui, ready, format!("{}  Upload to keyboard", icons::UPLOAD));
                    if r.clicked() {
                        let anim = self.screen.anim.take().unwrap();
                        self.screen.progress = Some((0, 1));
                        self.worker.send(Job::Upload(Box::new(anim)));
                        self.load_image(ctx);
                    }
                    if self.device.is_none() {
                        hint(ui, "Connect the keyboard to upload.");
                    } else if self.screen.anim.is_none() {
                        hint(ui, "Choose an image first.");
                    }
                }
            }
        });
    }
}

fn dashed_rect(p: &egui::Painter, r: egui::Rect, stroke: Stroke) {
    let pts = [r.left_top(), r.right_top(), r.right_bottom(), r.left_bottom(), r.left_top()];
    p.extend(egui::Shape::dashed_line(&pts, stroke, 8.0, 6.0));
}

// ---------------------------------------------------------------- Settings

impl App {
    fn settings_page(&mut self, root: &mut egui::Ui) {
        egui::CentralPanel::default().frame(Frame::new().fill(theme::BG).inner_margin(Margin::symmetric(22, 0))).show(
            root,
            |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.set_max_width(760.0);
                    page_header(ui, icons::GEAR, "Settings", "Profile maintenance and information about the app.");
                    let name = self.store.current().name.clone();

                    card(ui, |ui| {
                        card_header(
                            ui,
                            icons::FLOPPY,
                            "Write profile to keyboard",
                            &format!("Sends every layer and colour of \"{name}\" again, e.g. after using another app."),
                        );
                        if ui
                            .add_enabled(
                                self.device.is_some(),
                                egui::Button::new(format!("{}  Write now", icons::UPLOAD)),
                            )
                            .clicked()
                        {
                            self.write_profile();
                        }
                    });
                    ui.add_space(8.0);

                    card(ui, |ui| {
                        card_header(
                            ui,
                            icons::RESTORE,
                            "Restore factory key mapping",
                            &format!("Resets all four layers and colour palettes of \"{name}\" and writes them."),
                        );
                        if !self.confirm_reset {
                            if ui.button(format!("{}  Restore factory…", icons::RESTORE)).clicked() {
                                self.confirm_reset = true;
                            }
                        } else {
                            ui.horizontal(|ui| {
                                let b = egui::Button::new(
                                    RichText::new(format!("{}  Yes, reset \"{name}\"", icons::WARNING))
                                        .color(Color32::WHITE),
                                )
                                .fill(theme::DANGER);
                                if ui.add(b).clicked() {
                                    *self.store.current() = Profile::new(&name);
                                    self.save();
                                    self.write_profile();
                                    self.confirm_reset = false;
                                }
                                if ui.button("Cancel").clicked() {
                                    self.confirm_reset = false;
                                }
                            });
                        }
                    });
                    ui.add_space(8.0);

                    card(ui, |ui| {
                        card_header(
                            ui,
                            icons::STACK,
                            "Profile storage",
                            "The keyboard can't report its mapping, so this app keeps the master copy here.",
                        );
                        let path = profile::path().map_or("unavailable".into(), |p| p.display().to_string());
                        ui.label(RichText::new(path).monospace().color(theme::TEXT_MUTED));
                    });
                    ui.add_space(8.0);

                    card(ui, |ui| {
                        card_header(
                            ui,
                            icons::INFO,
                            &format!("Open S85 {}", env!("CARGO_PKG_VERSION")),
                            "Unofficial, open-source configuration tool for the Royal Kludge RK S85.",
                        );
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icons::GITHUB).size(16.0));
                            ui.hyperlink_to(
                                "github.com/Sergi-Morant/open-s85",
                                "https://github.com/Sergi-Morant/open-s85",
                            );
                        });
                        ui.label(
                            RichText::new(format!("{}  {} keys in the layout database", icons::KEYBOARD, KEYS.len()))
                                .color(theme::TEXT_MUTED),
                        );
                        ui.label(
                            RichText::new(format!(
                                "{}  Not affiliated with or endorsed by Royal Kludge. Use at your own risk.",
                                icons::SHIELD_WARNING
                            ))
                            .color(theme::TEXT_MUTED),
                        );
                    });
                    ui.add_space(16.0);
                });
            },
        );
    }
}
