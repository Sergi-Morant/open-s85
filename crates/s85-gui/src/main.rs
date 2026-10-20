#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod keyboard_view;
mod profile;
mod theme;
mod worker;

use std::path::PathBuf;

use eframe::egui::{self, Color32, RichText};
use s85_protocol::keycode::{catalog, Category, KeyCode};
use s85_protocol::keymap::Layer;
use s85_protocol::layout::{key_at_slot, KEYS};
use s85_protocol::lighting::{self, Config};
use s85_protocol::screen::{Animation, Fit};

use keyboard_view::KeyLook;
use profile::{Profile, Store};
use worker::{Event, Job, Worker};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Open S85")
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 600.0]),
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
    const ALL: [(Page, &'static str, &'static str); 4] = [
        (Page::Keys, "⌨", "Keys"),
        (Page::Lighting, "💡", "Lighting"),
        (Page::Screen, "🖼", "Screen"),
        (Page::Settings, "⚙", "Settings"),
    ];
}

#[derive(Default)]
struct ScreenState {
    path: Option<PathBuf>,
    fit: Fit,
    delay: Option<u16>,
    anim: Option<Animation>,
    preview: Option<egui::TextureHandle>,
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
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> App {
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
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();

        egui::Panel::left("nav").exact_size(76.0).resizable(false).show(root, |ui| {
            ui.add_space(12.0);
            for (page, icon, title) in Page::ALL {
                let active = self.page == page;
                let color = if active { theme::ACCENT } else { ui.visuals().text_color() };
                let r = ui
                    .add_sized(
                        [64.0, 56.0],
                        egui::Button::new(RichText::new(format!("{icon}\n{title}")).size(13.0).color(color))
                            .frame(false),
                    )
                    .on_hover_text(title);
                if r.clicked() {
                    self.page = page;
                }
                ui.add_space(6.0);
            }
        });

        egui::Panel::bottom("status").show(root, |ui| {
            ui.horizontal(|ui| {
                let (dot, label) = match &self.device {
                    Some(n) => (Color32::from_rgb(60, 200, 90), n.clone()),
                    None => (Color32::from_rgb(200, 60, 60), "Not connected".into()),
                };
                ui.colored_label(dot, "●");
                ui.label(label);
                if self.device.is_none() && ui.small_button("Retry").clicked() {
                    self.worker.send(Job::Connect);
                }
                ui.separator();
                let c = if self.status_is_error { theme::CHANGED } else { ui.visuals().weak_text_color() };
                ui.colored_label(c, &self.status);
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

// ---------------------------------------------------------------- Keys

impl App {
    fn profiles_panel(&mut self, root: &mut egui::Ui) {
        egui::Panel::left("profiles").exact_size(220.0).resizable(false).show(root, |ui| {
            ui.add_space(8.0);
            ui.heading("Profile");
            ui.horizontal(|ui| {
                if ui.button("➕").on_hover_text("New profile").clicked() {
                    let n = self.store.profiles.len() + 1;
                    self.store.profiles.push(Profile::new(&format!("Profile{n}")));
                    self.save();
                }
                if ui.button("🗑").on_hover_text("Delete profile").clicked() && self.store.profiles.len() > 1 {
                    self.store.profiles.remove(self.store.active);
                    self.store.active = self.store.active.saturating_sub(1);
                    self.save();
                }
                if ui.button("✏").on_hover_text("Rename profile").clicked() {
                    self.renaming = Some(self.store.current().name.clone());
                }
            });
            if let Some(name) = &mut self.renaming {
                let r = ui.text_edit_singleline(name);
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
            ui.separator();
            let mut switch_to = None;
            for (i, p) in self.store.profiles.iter().enumerate() {
                if ui.selectable_label(i == self.store.active, &p.name).clicked() && i != self.store.active {
                    switch_to = Some(i);
                }
            }
            if let Some(i) = switch_to {
                self.store.active = i;
                self.save();
                let p = self.store.current().clone();
                self.worker.send(Job::WriteAll(Box::new(p.keymap()), p.palettes()));
            }
        });
    }

    fn keys_page(&mut self, root: &mut egui::Ui) {
        self.profiles_panel(root);
        egui::CentralPanel::default().show(root, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                for l in Layer::ALL {
                    if ui.radio(self.layer == l, l.title()).clicked() {
                        self.layer = l;
                    }
                }
            });
            ui.add_space(6.0);
            let km = self.store.current().keymap();
            let factory = s85_protocol::keymap::Keymap::default();
            let layer = self.layer;
            let sel: Vec<u8> = self.selected.into_iter().collect();
            let clicked = keyboard_view::show(ui, &sel, |k| {
                let code = km.get(layer, k.slot);
                let empty = code == KeyCode::NONE && layer != Layer::Default;
                KeyLook {
                    text: if empty { keyboard_view::short(k.label) } else { keyboard_view::short(&code.name()) },
                    dim: empty,
                    changed: code != factory.get(layer, k.slot),
                    fill: None,
                }
            });
            if let Some(s) = clicked {
                self.selected = if self.selected == Some(s) { None } else { Some(s) };
            }
            ui.add_space(8.0);
            match self.selected.and_then(key_at_slot) {
                Some(k) => {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{} on {}:", k.label, layer.title())).strong());
                        ui.label(km.get(layer, k.slot).name());
                        let slot = k.slot;
                        if ui.button("Factory").on_hover_text("Restore this key's factory function").clicked() {
                            self.assign(slot, factory.get(layer, slot));
                        }
                    });
                }
                None => {
                    ui.weak("Click a key, then choose its new function below.");
                }
            }
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| self.key_picker(ui));
        });
    }

    fn key_picker(&mut self, ui: &mut egui::Ui) {
        let mut pick = None;
        let enabled = self.selected.is_some();
        let ncols = ((ui.available_width() / 270.0).floor() as usize).clamp(1, Category::ALL.len());
        for row in Category::ALL.chunks(ncols) {
            ui.columns(ncols, |cols| {
                for (col, &cat) in cols.iter_mut().zip(row) {
                    col.label(RichText::new(cat.title()).strong().size(15.0).color(theme::ACCENT));
                    col.horizontal_wrapped(|ui| {
                        for e in catalog().iter().filter(|e| e.category == cat) {
                            if ui.add_enabled(enabled, egui::Button::new(e.name)).clicked() {
                                pick = Some(e.code);
                            }
                        }
                    });
                }
            });
            ui.add_space(12.0);
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

// ---------------------------------------------------------------- Lighting

impl App {
    fn lighting_page(&mut self, root: &mut egui::Ui) {
        let Some(mut cfg) = self.cfg.clone() else {
            egui::CentralPanel::default().show(root, |ui| {
                ui.add_space(40.0);
                ui.vertical_centered(|ui| {
                    ui.label("Lighting settings are read from the keyboard.");
                    if ui.button("Connect").clicked() {
                        self.worker.send(Job::Connect);
                    }
                });
            });
            return;
        };
        let before = cfg.clone();
        let mut palette_changed = false;

        egui::Panel::left("modes").exact_size(220.0).resizable(false).show(root, |ui| {
            ui.add_space(8.0);
            ui.heading("Light Effect Mode");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for &(m, name) in lighting::MODES.iter().chain([(lighting::MODE_OFF, "OFF")].iter()) {
                    if ui.radio(cfg.mode() == m, name).clicked() {
                        cfg.set_mode(m);
                    }
                }
            });
        });

        egui::CentralPanel::default().show(root, |ui| {
            let m = cfg.mode();
            let mut pal = self.store.current().palettes();
            let color = cfg.color(m);
            let rgb = pal.0[m as usize][(color as usize).min(6)];
            ui.add_space(6.0);
            keyboard_view::show(ui, &[], |k| KeyLook {
                text: keyboard_view::short(k.label),
                dim: false,
                changed: false,
                fill: (m != lighting::MODE_OFF && color != lighting::COLOR_MIX && lighting::mode_has_color(m))
                    .then(|| Color32::from_rgb(rgb[0], rgb[1], rgb[2]).gamma_multiply(0.85)),
            });
            ui.add_space(12.0);
            if m == lighting::MODE_OFF {
                ui.label("Lighting is off.");
                return;
            }
            ui.columns(2, |cols| {
                let ui = &mut cols[0];
                ui.label(RichText::new("Light Brightness").size(16.0));
                let mut b = cfg.brightness(m);
                let r = ui.add(
                    egui::Slider::new(&mut b, 0..=lighting::MAX_BRIGHTNESS)
                        .step_by(lighting::BRIGHTNESS_STEP as f64),
                );
                if r.changed() {
                    cfg.set_brightness(m, b);
                }
                if lighting::mode_has_speed(m) {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Animation Speed").size(16.0));
                    let mut s = cfg.speed(m);
                    if ui.add(egui::Slider::new(&mut s, 0..=lighting::MAX_SPEED)).changed() {
                        cfg.set_speed(m, s);
                    }
                }
                ui.add_space(10.0);
                ui.label(RichText::new("Wireless sleep time").size(16.0));
                let cur = cfg.sleep_minutes();
                let label = |v: u8| if v == 0 { "Never".to_string() } else { format!("{v} min") };
                egui::ComboBox::from_id_salt("sleep").selected_text(label(cur)).show_ui(ui, |ui| {
                    for v in lighting::SLEEP_MINUTES {
                        if ui.selectable_label(cur == v, label(v)).clicked() {
                            cfg.set_sleep_minutes(v);
                        }
                    }
                });

                let ui = &mut cols[1];
                if !lighting::mode_has_color(m) {
                    ui.weak("This effect uses its own colours.");
                    return;
                }
                let mut mix = color == lighting::COLOR_MIX;
                if ui.checkbox(&mut mix, "Color Mixing").changed() {
                    cfg.set_color(m, if mix { lighting::COLOR_MIX } else { 0 });
                }
                ui.add_space(6.0);
                ui.label(RichText::new("Color Panel").size(16.0));
                let mut c = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                let mut new_rgb = None;
                if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque)
                    .changed()
                {
                    new_rgb = Some([c.r(), c.g(), c.b()]);
                }
                ui.horizontal(|ui| {
                    for p in lighting::DEFAULT_PALETTE {
                        let (r, resp) = ui.allocate_exact_size(egui::vec2(24.0, 24.0), egui::Sense::click());
                        ui.painter().rect_filled(r, 3.0, Color32::from_rgb(p[0], p[1], p[2]));
                        if resp.clicked() {
                            new_rgb = Some(p);
                        }
                    }
                });
                if let Some(v) = new_rgb {
                    // Like the official app: the chosen colour goes into palette slot 0 and is selected.
                    pal.0[m as usize][0] = v;
                    cfg.set_color(m, 0);
                    self.store.current().set_palettes(&pal);
                    palette_changed = true;
                }
            });
        });

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

// ---------------------------------------------------------------- Screen

impl App {
    fn load_image(&mut self, ctx: &egui::Context) {
        let Some(path) = self.screen.path.clone() else { return };
        match Animation::load(&path, self.screen.fit, self.screen.delay) {
            Ok(anim) => {
                let f = &anim.frames[0];
                let img = egui::ColorImage::from_rgb([f.width() as usize, f.height() as usize], f.as_raw());
                self.screen.preview = Some(ctx.load_texture("preview", img, Default::default()));
                self.screen.anim = Some(anim);
            }
            Err(e) => self.error(format!("Could not load image: {e}")),
        }
    }

    fn screen_page(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        let ctx = &ctx;
        egui::CentralPanel::default().show(root, |ui| {
            ui.add_space(8.0);
            ui.heading("TFT screen");
            ui.add_space(8.0);
            let mut reload = false;
            ui.horizontal(|ui| {
                if ui.button("Choose image or GIF…").clicked() {
                    if let Some(p) = rfd::FileDialog::new()
                        .add_filter("Images", &["gif", "png", "jpg", "jpeg", "bmp", "webp"])
                        .pick_file()
                    {
                        self.screen.path = Some(p);
                        reload = true;
                    }
                }
                if let Some(p) = &self.screen.path {
                    ui.label(p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
                }
            });
            ui.horizontal(|ui| {
                ui.label("Fit:");
                for (f, name) in [(Fit::Cover, "Fill (crop)"), (Fit::Contain, "Fit (bars)"), (Fit::Stretch, "Stretch")] {
                    if ui.radio(self.screen.fit == f, name).clicked() {
                        self.screen.fit = f;
                        reload = true;
                    }
                }
            });
            ui.horizontal(|ui| {
                let mut custom = self.screen.delay.is_some();
                if ui.checkbox(&mut custom, "Custom frame delay").changed() {
                    self.screen.delay = custom.then_some(100);
                    reload = true;
                }
                if let Some(d) = &mut self.screen.delay {
                    if ui.add(egui::DragValue::new(d).range(10..=5000).suffix(" ms")).changed() {
                        reload = true;
                    }
                }
            });
            if reload {
                self.load_image(ctx);
            }
            ui.add_space(10.0);
            if let (Some(tex), Some(anim)) = (&self.screen.preview, &self.screen.anim) {
                ui.image((tex.id(), egui::vec2(320.0, 172.0) * 1.5));
                let secs = anim.frames.len() as f32 * s85_protocol::screen::CHUNKS_PER_FRAME as f32 * 0.073;
                ui.label(format!(
                    "{} frame(s), {} ms per frame, about {:.0} s to upload",
                    anim.frames.len(),
                    anim.delay_ms,
                    secs.max(1.0)
                ));
                ui.add_space(6.0);
                match self.screen.progress {
                    Some((done, total)) => {
                        ui.add(egui::ProgressBar::new(done as f32 / total as f32).show_percentage());
                    }
                    None => {
                        if ui.add_enabled(self.device.is_some(), egui::Button::new("Upload to keyboard")).clicked() {
                            let anim = self.screen.anim.take().unwrap();
                            self.screen.progress = Some((0, 1));
                            self.worker.send(Job::Upload(Box::new(anim)));
                            self.load_image(ctx);
                        }
                    }
                }
            }
            ui.add_space(16.0);
            ui.separator();
            if ui.button("Sync clock with this computer").clicked() {
                self.worker.send(Job::SyncClock);
            }
        });
    }
}

// ---------------------------------------------------------------- Settings

impl App {
    fn settings_page(&mut self, root: &mut egui::Ui) {
        egui::CentralPanel::default().show(root, |ui| {
            ui.add_space(8.0);
            ui.heading("Settings");
            ui.add_space(8.0);
            if ui.button("Write current profile to keyboard").clicked() {
                let p = self.store.current().clone();
                self.worker.send(Job::WriteAll(Box::new(p.keymap()), p.palettes()));
            }
            ui.add_space(4.0);
            if ui
                .button("Restore factory key mapping (this profile)")
                .on_hover_text("Resets all four layers and colour palettes of the current profile")
                .clicked()
            {
                let name = self.store.current().name.clone();
                *self.store.current() = Profile::new(&name);
                self.save();
                let p = self.store.current().clone();
                self.worker.send(Job::WriteAll(Box::new(p.keymap()), p.palettes()));
            }
            ui.add_space(20.0);
            ui.separator();
            ui.label(RichText::new("About").strong());
            ui.label(format!("Open S85 {}", env!("CARGO_PKG_VERSION")));
            ui.label("Unofficial, open-source configuration tool for the Royal Kludge RK S85.");
            ui.label("Not affiliated with or endorsed by Royal Kludge. Use at your own risk.");
            ui.label(format!("Keys known: {}", KEYS.len()));
        });
    }
}
